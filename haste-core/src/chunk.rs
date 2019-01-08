use crate::offer::Offer;

use failure::{format_err, Error};

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::time::{self, SystemTime};

pub fn chunk_it(
    num: usize,
    cpu_percent: usize,
    memory: usize,
    offers: &[Offer],
) -> Result<Chunks, Error> {
    if num % 2 != 0 {
        return Err(format_err!("master number must be even"));
    }
    if offers.len() < 3 {
        return Err(format_err!("agent must more than 3"));
    }
    if offers.len() == 3 && num == 4 {
        return Err(format_err!("can not deploy 4 master node with 3 host"));
    }

    let mut ics = into_count(cpu_percent, memory, offers);
    let ports_map: HashMap<String, VecDeque<usize>> = offers
        .into_iter()
        .map(|x| {
            let mut ports = x.ports.clone();
            ports.sort();
            (x.host.clone(), ports.into_iter().collect())
        })
        .collect();

    let total_count: usize = ics.iter().map(|x| x.count).sum();
    if total_count < num * 2 {
        return Err(format_err!("not enough resource. plz call administractor"));
    }
    ics.sort_by(|x, y| x.count.cmp(&y.count));
    let nics = dp_fill(&ics, num * 2, 2);
    if check_dist(&nics, num * 2) {
        return Err(format_err!(
            "max instance is more than half nodes of the cluster"
        ));
    }

    let hcount = nics.len();

    let link_pos: HashMap<String, usize> =
        ics.iter().map(|ic| (ic.host.clone(), ic.count)).collect();
    let mut link_table: Vec<Vec<usize>> = (0..hcount)
        .map(|_| (0..hcount).map(|_| 0).collect())
        .collect();
    let mut links: Vec<Link> = Vec::new();

    loop {
        let ic = max_count_ic(&ics);
        if ic.count == 0 {
            break;
        }
        let pos = link_pos[&ic.host];
        let llh = find_min_link(&link_table, pos);
        if ics[llh].count < 2 {
            link_table[llh][pos] += 1;
            link_table[pos][llh] += 1;
            continue;
        }
        links.push(Link {
            base: ic.host.clone(),
            link_to: ics[llh].host.clone(),
        });
        link_table[llh][pos] += 1;
        link_table[pos][llh] += 1;
        ics[pos].count -= 2;
        ics[llh].count -= 2;
    }
    let chunks = links2chunks(links, ports_map);
    Ok(chunks)
}

pub const ROLE_MASTER: &str = "master";
pub const ROLE_SLAVE: &str = "slave";

fn links2chunks(links: Vec<Link>, mut ports_map: HashMap<String, VecDeque<usize>>) -> Chunks {
    let mut chunks = Chunks(Vec::new());
    let now = SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap();
    let mut runid_term = now.as_secs() << 20;
    let num = links.len() * 2; // master number

    for link in links {
        let p1 = ports_map.get_mut(&link.base).unwrap().pop_front().unwrap();
        let p2 = ports_map.get_mut(&link.base).unwrap().pop_front().unwrap();
        let p3 = ports_map
            .get_mut(&link.link_to)
            .unwrap()
            .pop_front()
            .unwrap();
        let p4 = ports_map
            .get_mut(&link.link_to)
            .unwrap()
            .pop_front()
            .unwrap();

        runid_term += 1;
        let runid1 = format!("{:040}", runid_term);
        runid_term += 1;
        let runid2 = format!("{:040}", runid_term);
        runid_term += 1;
        let runid3 = format!("{:040}", runid_term);
        runid_term += 1;
        let runid4 = format!("{:040}", runid_term);

        let instances = vec![
            Instance {
                host: link.base.clone(),
                port: p1,
                role: ROLE_MASTER.to_string(),
                slaveof: "-".to_string(),
                runid: runid1.clone(),
                slots: Vec::new(),
            },
            Instance {
                host: link.base.clone(),
                port: p2,
                role: ROLE_SLAVE.to_string(),
                slaveof: runid3.clone(),
                runid: runid2,
                slots: Vec::new(),
            },
            Instance {
                host: link.link_to.clone(),
                port: p3,
                role: ROLE_MASTER.to_string(),
                slaveof: "-".to_string(),
                runid: runid3,
                slots: Vec::new(),
            },
            Instance {
                host: link.link_to.clone(),
                port: p4,
                role: ROLE_SLAVE.to_string(),
                slaveof: runid1.clone(),
                runid: runid4,
                slots: Vec::new(),
            },
        ];
        chunks.0.extend(instances);
    }

    let per = 16384 / num;
    let left = 16384 % num;
    let mut base = 0;
    let mut cursor = 0;

    for inst in chunks.0.iter_mut() {
        let mut scount = per;
        if cursor < left {
            scount += 1;
        }
        let end = if base + scount >= 16384 {
            16383
        } else {
            base + scount
        };

        inst.slots.push(Slot { begin: base, end });
        base = end + 1;
        cursor += 1;
    }
    chunks
}

fn find_min_link(link_table: &Vec<Vec<usize>>, pos: usize) -> usize {
    let row = &link_table[pos];
    let mut cursor = if pos == 0 { 1 } else { 0 };

    for &rp in row {
        if pos == rp {
            continue;
        }

        if row[rp] < row[cursor] {
            cursor = pos
        }
    }
    cursor
}

fn max_count_ic(ics: &[InstanceCount]) -> &InstanceCount {
    let mut count = 0;
    let mut i = 0;
    for (idx, ic) in ics.iter().enumerate() {
        if count < ic.count {
            count = ic.count;
            i = idx;
        }
    }
    &ics[i]
}

struct Link {
    base: String,
    link_to: String,
}

fn check_dist(ics: &[InstanceCount], total: usize) -> bool {
    for ic in ics {
        if ic.count >= total / 2 {
            return true;
        }
    }
    false
}

fn dp_fill(ics: &Vec<InstanceCount>, mut count: usize, scale: usize) -> Vec<InstanceCount> {
    let mut counter = ics.iter().map(|x| (x.host.clone(), 0)).collect();
    let ic_map: HashMap<String, usize> = ics.iter().map(|x| (x.host.clone(), x.count)).collect();
    while count != 0 {
        let name = find_min(&counter, &ic_map, scale);
        let handle = counter.entry(name.to_string()).or_insert_with(|| 0);
        *handle += scale;
        count -= scale;
    }
    counter
        .into_iter()
        .map(|(host, count)| InstanceCount { host, count })
        .collect()
}

fn find_min<'a>(
    counter: &'a HashMap<String, usize>,
    ic_map: &HashMap<String, usize>,
    scale: usize,
) -> &'a str {
    let mut min = std::usize::MAX;
    let mut host = "";
    for (h, &count) in counter.iter() {
        if count - ic_map[h] > scale {
            continue;
        }

        if count == 0 {
            return h;
        }

        if count < min {
            min = count;
            host = h;
        }
    }
    host
}

fn into_count(cpu_percent: usize, memory: usize, offers: &[Offer]) -> Vec<InstanceCount> {
    let mut ics = Vec::new();
    for offer in offers {
        let c = offer.cpu / cpu_percent;
        let m = offer.memory / memory;
        let p = offer.ports.len();
        let count = c.min(m).min(p) / 2 * 2;
        if count < 1 {
            continue;
        }
        ics.push(InstanceCount {
            host: offer.host.clone(),
            count: count,
        })
    }
    ics
}

#[derive(Clone, Debug)]
struct InstanceCount {
    host: String,
    count: usize,
}

pub struct Chunks(pub Vec<Instance>);

pub struct Instance {
    pub host: String,
    pub port: usize,
    pub role: String,

    pub slaveof: String,
    pub runid: String,
    pub slots: Vec<Slot>,
}

impl Instance {
    pub fn as_conf_line(&self, myself: bool) -> String {
        let flags = if myself {
            format!("myself,{}", self.role)
        } else {
            self.role.clone()
        };
        let slots: Vec<_> = self.slots.iter().map(|x| format!("{}", x)).collect();
        let slots_str = slots.join(" ");

        format!(
            "{} {}:{}@{} {} {} 0 0 0 connected {}\n",
            self.runid,
            self.host,
            self.port,
            self.port + 10000,
            flags,
            self.slaveof,
            slots_str,
        )
    }
}

pub struct Slot {
    begin: usize,
    end: usize,
}

impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.begin == self.end {
            write!(f, "{}", self.begin)
        } else {
            write!(f, "{}-{}", self.begin, self.end)
        }
    }
}
