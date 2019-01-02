use crate::proto::{CacheInfo, CacheType, SystemdAction};
use crate::systemd::do_action;

use failure::Error;
use log::{info, warn};
use reqwest;
use tempfile::tempdir;

use std::fs::{self, File};
use std::io::{Read, Write};
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
    fn clean_service_dirty(&mut self, port: i64) -> Result<bool, Error> {
        let mut exists = false;
        info!("tryint to check if instance on port {} exists", port);
        let mut inst_pb = PathBuf::from(INSTANCE_DIR);
        inst_pb.push(&format!("{}", port));
        if inst_pb.as_path().exists() {
            fs::remove_dir_all(inst_pb.as_path())?;
            exists = true;
        }

        let mut systemd_pb = PathBuf::from(SYSTEMD_DIR);
        systemd_pb.push(&format!("cache@{}.service", port));
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

    fn clean_dirty(&mut self) -> Result<(), Error> {
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

    fn check_binary(&mut self) -> Result<bool, Error> {
        let cache_type = self.ci.get_cache_type();
        let version = self.ci.get_version().to_string();

        let mut lib_dir = PathBuf::from(LIB_DIR);
        lib_dir.push(cache_type_as_str(cache_type));
        lib_dir.push(version);
        Ok(lib_dir.as_path().exists())
    }

    fn setup_binary(&mut self) -> Result<(), Error> {
        let fs = self.ci.get_file_server().to_string();
        let version = self.ci.get_version().to_string();
        let cache_type = cache_type_as_str(self.ci.get_cache_type());
        let url = format!("{}/{}-{}.tar.gz", fs, cache_type, version);

        let dir = tempdir()?;
        let path = dir.path().join(format!("{}-{}.tar.gz", cache_type, version));
        self.download_binary(path, &url)?;

        Ok(())
    }

    fn extract_gzip<T: AsRef<Path>>(&mut self, file: T, dir: T) -> Result<(), Error> {
        // decompress tar.gz into tar with gzip file

        // extract tar file into given file

        unimplemented!()
    }

    fn download_binary<T: AsRef<Path>>(&mut self, fpath: T, url: &str) -> Result<(), Error> {
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

    fn render_files(&mut self) -> Result<(), Error> {
        unimplemented!();
    }

    fn setup_systemd(&mut self) -> Result<(), Error> {
        unimplemented!();
    }

    fn spawn_cache(&mut self) -> Result<(), Error> {
        unimplemented!();
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
