use crate::proto::{CacheInfo, CacheType, SystemdAction};
use crate::systemd::{do_action, service_name};

use failure::Error;
use log::{debug, info, warn};
use reqwest;
use tempfile::tempdir;

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub const LIB_DIR: &str = "/data/haste/lib";
pub const INSTANCE_DIR: &str = "/data/haste/instance";
pub const SYSTEMD_DIR: &str = "/etc/systemd/";

pub struct CacheDeployer {
    ci: CacheInfo,
    locker: Arc<Mutex<()>>,
}

impl CacheDeployer {
    pub fn new(ci: CacheInfo, locker: Arc<Mutex<()>>) -> Self {
        CacheDeployer { ci, locker }
    }

    /// deploy cache instances
    ///
    ///   0. clean dirty dir
    ///   1. check if binary exists
    ///       1.1 if not exists
    ///       1.2 flock binary lock (defer unlock)
    ///       1.2 download binary file
    ///   2. redner files
    ///   3. setup systemd service
    ///   4. spwan systemd service
    ///
    pub fn deploy(&mut self) -> Result<(), Error> {
        info!("start deploy to {:?}", self.ci);

        info!("trying to clean dirty service files");
        self.clean_dirty()?;

        if self.check_binary()? {
            info!("download and setup cache binary files");
            self.setup_binary()?;
        }

        info!("render config files");
        self.render_files()?;
        info!("setup systemd service");
        self.setup_systemd()?;
        info!("spawn cache service");
        self.spawn_cache()?;
        info!("all terms is done, good luck.");

        Ok(())
    }

    // check if files and delete them:
    //   1. /data/cache/instance/{port} exists
    //   2. if /etc/systemd/cache@{port}.service file exists
    fn clean_service_dirty(&self, port: i64) -> Result<bool, Error> {
        let mut exists = false;
        info!("tryint to check if instance on port {} exists", port);
        let mut inst_pb = PathBuf::from(INSTANCE_DIR);
        inst_pb.push(&format!("{}", port));
        if inst_pb.as_path().exists() {
            fs::remove_dir_all(inst_pb.as_path())?;
            exists = true;
        }

        let mut systemd_pb = PathBuf::from(SYSTEMD_DIR);
        systemd_pb.push(&service_name(port));
        if systemd_pb.as_path().exists() {
            if let Err(err) = do_action(SystemdAction::Stop, port) {
                warn!("the service is not running but ignore it. error: {:?}", err);
            }
            if let Err(err) = do_action(SystemdAction::Remove, port) {
                warn!("remove service not done but ignore it by error {:?}", err);
            }
            exists = true;
        }

        Ok(exists)
    }

    fn clean_dirty(&self) -> Result<(), Error> {
        let ports: Vec<_> = self
            .ci
            .get_insts()
            .into_iter()
            .map(|x| x.get_port())
            .collect();
        for port in ports {
            self.clean_service_dirty(port)?;
        }
        Ok(())
    }

    fn check_binary(&self) -> Result<bool, Error> {
        let cache_type = self.ci.get_cache_type();
        let version = self.ci.get_version().to_string();

        let mut lib_dir = PathBuf::from(LIB_DIR);
        lib_dir.push(cache_type_as_str(cache_type));
        lib_dir.push(version);
        lib_dir.push(cache_type_as_binary_name(cache_type));
        Ok(lib_dir.as_path().exists())
    }

    fn setup_binary(&self) -> Result<(), Error> {
        let fs = self.ci.get_file_server().to_string();
        let version = self.ci.get_version().to_string();
        let cache_type = cache_type_as_str(self.ci.get_cache_type());
        let bname = cache_type_as_binary_name(self.ci.get_cache_type());

        let url = format!("{}/{}-{}-{}", fs, cache_type, version, bname);

        let dir = tempdir()?;
        let path = dir
            .path()
            .join(format!("{}-{}-{}", cache_type, version, bname));
        self.download_binary(&path, &url)?;

        // rename
        // make dir first
        let mut real_pb = PathBuf::from(LIB_DIR);
        real_pb.push(&cache_type);
        real_pb.push(&version);

        fs::create_dir_all(real_pb.as_path())?;

        real_pb.push(bname);
        let real_path = real_pb.as_path();
        let _guard = self.locker.lock();
        fs::rename(&path, real_path)?;

        if cfg!(not(windows)) {
            use std::fs::Permissions;
            use std::os::unix::fs::PermissionsExt;

            let bfile = File::open(real_path)?;
            bfile.set_permissions(Permissions::from_mode(0755))?;
        }

        Ok(())
    }

    fn download_binary<T: AsRef<Path>>(&self, fpath: T, url: &str) -> Result<(), Error> {
        let mut file = File::create(fpath.as_ref())?;
        let mut response = reqwest::get(url)?;
        let size = response.copy_to(&mut file)?;
        info!(
            "download succeed to file {:?} with {} bytes",
            fpath.as_ref(),
            size
        );
        Ok(())
    }

    fn render_files(&self) -> Result<(), Error> {
        let files = self
            .ci
            .get_insts()
            .into_iter()
            .map(|y| y.get_files().into_iter().map(|x| x.clone()))
            .flatten();

        for file in files {
            debug!(
                "create file {} with content {}",
                file.get_fpath(),
                file.get_content()
            );
            let path = Path::new(file.get_fpath());
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir)?;
            }

            let mut fp = File::create(path)?;
            fp.write_all(file.get_content().as_bytes())?;
        }

        Ok(())
    }

    fn setup_systemd(&self) -> Result<(), Error> {
        let _guard = self.locker.lock();
        do_action(SystemdAction::Setup, -1)?;
        Ok(())
    }

    fn spawn_cache(&self) -> Result<(), Error> {
        let ports = self.ci.get_insts().into_iter().map(|x| x.get_port());
        for port in ports {
            do_action(SystemdAction::Start, port)?;
        }
        Ok(())
    }
}

fn cache_type_as_binary_name(cache_type: CacheType) -> &'static str {
    match cache_type {
        CacheType::Memcache => "memcached",
        _ => "redis-server",
    }
}

fn cache_type_as_str(cache_type: CacheType) -> &'static str {
    const CACHE_TYPE_REDIS: &str = "redis";
    const CACHE_TYPE_MEMCACHE: &str = "memcache";

    match cache_type {
        CacheType::Memcache => CACHE_TYPE_MEMCACHE,
        _ => CACHE_TYPE_REDIS,
    }
}
