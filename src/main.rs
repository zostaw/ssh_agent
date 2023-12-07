use ssh2::{Channel, Session};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};

use serde::{Deserialize, Serialize};

// defines how often to sleep between ssh calls
const DELAY: std::time::Duration = std::time::Duration::from_millis(100);
const DB_NAME: &str = "my_db";

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
struct Process {
    username: Option<String>,
    private_key_path: Option<String>,
    ip: String,
    port: Option<String>,
    command: String,
}

#[allow(dead_code)]
impl Process {
    fn new(
        ip: String,
        username: Option<String>,
        private_key_path: Option<String>,
        port: Option<String>,
        command: String,
    ) -> Process {
        return Process {
            ip,
            username,
            private_key_path,
            port,
            command,
        };
    }

    fn update_db(&self, hostname: &String, ip: &String, version: &String) {
        println!("Update DB");

        let connection = sqlite::open(DB_NAME).unwrap();

        // let mut statement = connection.prepare(query).unwrap();
        // statement.bind::<(_, Err)>((":name", ip.into())).unwrap();
        // statement.bind((":version", version.into())).unwrap();

        let query = format!(
            "
            CREATE TABLE IF NOT EXISTS {} (ip TEXT, version TEXT);
            INSERT INTO {} VALUES ('{}', '{}');
            ",
            &hostname, &hostname, &ip, &version
        );
        // let query = "SELECT * FROM users WHERE name = :name";
        // let mut statement = connection.prepare(query).unwrap();
        // statement.bind(&[(":name", "Bob")][..]).unwrap();
        connection.execute(query).unwrap();
    }

    fn trigger_update(&self, sess: &mut Session) -> Result<String, Box<dyn std::error::Error>> {
        let mut s: String = Default::default();
        // let command = String::from("ls -last");

        let hostname = String::from("hostname");
        let mut hostname_out: String = Default::default();
        let mut channel = sess.channel_session().unwrap();
        channel.exec(&hostname)?;
        channel.read_to_string(&mut hostname_out)?;

        let ip = String::from("ip a | grep inet | grep -v inet6 | grep -v 127.0.0.1");
        let mut ip_out: String = Default::default();
        let mut channel = sess.channel_session().unwrap();
        channel.exec(&ip)?;
        channel.read_to_string(&mut ip_out)?;

        let version = String::from("uname -a");
        let mut version_out: String = Default::default();
        let mut channel = sess.channel_session().unwrap();
        channel.exec(&version)?;
        channel.read_to_string(&mut version_out)?;

        Self::update_db(&self, &hostname_out, &ip_out, &version_out);
        //                 sshpass -e ssh -o StrictHostKeyChecking=no kplus@$ZONE "echo -n zone_ip:: ; $GET_IP ;
        //                                             echo -n zone_kplusver:: ; $GET_KPLUS_VER ;
        //                                             echo -n zone_kplustp:: ; if [ -e $KPLUSTP_DIR ]; then echo K+TP_YES ; else echo K+TP_NO ; fi ;
        //                                             echo -n zone_kplustp_ver:: ; $GET_KPLUSTP_VER ;
        //                                             echo -n zone_kgr:: ; if [ -e $KGR_DIR ]; then echo KGR_YES ; else echo KGR_NO ; fi ;
        //                                             echo -n zone_kgr_ver:: ; $GET_KGR_VER ;
        //                                             echo -n zone_os:: ; $GET_OS ;
        //                                             echo -n zone_os_ver:: ; $GET_OS_VERSION ;
        //                                             echo -n zone_dataserver:: ; $GET_DB ;
        //                                             if [[ \$(eval $GET_DATASERVER) = 'SQL_SERVER' ]] ; then echo -n zone_dbserver_ip:: ; $GET_DBSERVER_IP ; fi ;
        //                                             echo -n zone_els_ver:: ; $GET_ELS_VERSION ;
        //                                             echo -n zone_els_exp:: ; $GET_ELS_EXPIRATION ;
        //                                             echo -n zone_rvd_ver:: ; $GET_RVD_VERSION ;
        //                                             echo -n zone_python_ver:: ; $GET_PYTHON_VER ;
        //                                             echo -n zone_java_ver:: ; $GET_JAVA_VERSION"
        //                 sshpass -e ssh -T kplus@$ZONE "echo -n zone_dbdate:: ; $GET_DB_DATE ;
        //                                             echo -n zone_dbver:: ; $GET_DB_VERSION"
        // #this next part is not pretty, it is necesarry
        //                 sshpass -e ssh kplus@$ZONE "echo -n zone_kms:: ; if [[ -n \$(eval $kmsPID) ]]; then echo KMS_ON ; else echo KMS_OFF ; fi ;
        // 			echo -n zone_klws:: ; if [[ -n \$(eval $klwsPID) ]]; then echo KLWS_ON ; else echo KLWS_OFF ; fi ;
        // 			echo -n zone_kdews:: ; if [[ -n \$(eval $kdewsPID) ]]; then echo KDEWS_ON ; else echo KDEWS_OFF ; fi ;
        // 			echo -n zone_krth:: ; if [[ -n \$(eval $krthPID) ]]; then echo KRTH_ON ; else echo KRTH_OFF ; fi ;
        // 			echo -n zone_realtime:: ; if [[ -n \$(eval $realtimeSERVER) ]];
        // 				then if [ $( printf '$('"$realtimeSERVERstatus"')' ) = "Up" ] ;
        // 					then echo $( printf '$('"$realtimeSERVERname"')_ON' ) ; else echo $( printf '$('"$realtimeSERVERname"')_OFF' ) ; fi
        // 				else echo REALTIME_NO ; fi ;
        //             if [[ \$(eval $realtimeSERVERname) = 'RTMD' ]] ; then echo -n zone_rtmd_ver:: ; $GET_RTMD_VERSION ; fi ;
        // 			echo -n zone_idn:: ; if [[ -n \$(eval $realtimeIDN) ]]; then if [ $( printf '$('"$realtimeIDNstatus"')' ) = "Up" ] ; then echo IDN_ON ; else echo IDN_OFF ; fi else echo IDN_OFF ; fi "

        // export GET_KPLUS_VER="cat dist/VERSION.kplus | awk '{print \$4}'"
        // GET_KPLUSTP_VER="if [ -e /kenvng/home/kplustp ]; then cat /kenvng/home/kplustp/dist/VERSION.ktpplus | sed 's/ //g' ; else echo K+TP_NO ; fi"
        // export GET_OS="if [ -e /etc/redhat-release ] ; then echo Linux ; else echo Solaris ; fi"
        // export GET_OS_VERSION="if [ -e /etc/release ]; then uname -r; else cat /etc/redhat-release | sed 's/ /_/g' ; fi"
        // export GET_DB="if grep DBMS_TYPE /kenvng/home/kplus/dist/entities/Standalone/config/kplus.params > /dev/null ; then
        //                 grep DBMS_TYPE /kenvng/home/kplus/dist/entities/Standalone/config/kplus.params | grep -v ENTITY | awk '{print \$2}' ; else echo unknown ; fi"
        // export KPLUSTP_DIR="/kenvng/home/kplustp"
        // export KGR_DIR="/kenvng/home/kgr"
        // GET_KGR_VER="if [ -e $KGR_DIR ]; then cat /kenvng/home/kgr/VERSION.fusionrisk | sed 's/ //g' ; else echo KGR_NO ; fi"
        // GET_ELS_VERSION="if [ -e /kenvng/els/VERSION.els ]; then nawk 1 /kenvng/els/VERSION.els | sed 's/ /_/g' ; else echo unknown; fi"
        // GET_ELS_EXPIRATION="if [ -e /etc/release ]; then ggrep -m1 APPLICATION /kenvng/els/license.def | ggrep -o '[0-9]*/[0-9]*/[0-9]*' ; else grep -m1 APPLICATION /kenvng/els/license.def | grep -o '[0-9]*/[0-9]*/[0-9]*' ; fi"
        // GET_RVD_VERSION="if [ -e /kenvng/rvd/release_notes/ ]; then ls -l /kenvng/rvd/release_notes/ | grep -v html | grep TIB | awk '{print \$9}' | sed 's/_license.pdf//g' ; else echo unknown ; fi"
        // GET_PYTHON_VER="python -V 2>&1 | sed 's/ //g'"
        // GET_JAVA_VERSION="/kenvng/java/jdk/bin/java -version 2>&1 | grep version | sed 's/ //g' | sed 's/java//g'"
        // GET_RTMD_VERSION="cat /kenvng/home/kplus/rtmd/rtmd.version | grep RealTime | sed 's/[^0-9.]*//g'"

        // #variables for database queries
        // GET_DATE_LOCAL='date +%d/%m/%Y'
        // GET_DB_DATE="if [ -e /nfs/vol5/csrepository/getDBDate.sh ]; then /nfs/vol5/csrepository/getDBDate.sh ; else echo missing_repository ; fi"

        // GET_DATASERVER="grep DBMS_TYPE /kenvng/home/kplus/dist/entities/Standalone/config/kplus.params | grep -v ENTITY | awk '{print \$2}'"
        // GET_DBSERVER_IP="nslookup \$(grep DATASERVER_HOST /kenvng/home/kplus/dist/entities/Standalone/config/kplus.params | awk '{print \$2}') |
        //     grep -A2 Non-authoritative | grep Address | awk '{print \$2}'"
        // GET_DB_VERSION="if [ -e /nfs/vol5/csrepository/getDBVersion.sh ]; then /nfs/vol5/csrepository/getDBVersion.sh ; else echo missing_repository ; fi"

        // #boolean for determining if the zone is down or online
        // PROCESS_ZONE="FALSE"

        // #variables for the process statuses
        // kmsPID="ps -ef | grep kms | egrep -v 'ps|grep|startksrv|--agent' | awk '{print \$2;}'"
        // klwsPID="ps -ef | grep klws | egrep -v 'ps|grep|startksrv|--agent' | awk '{print \$2;}'"
        // kdewsPID="ps -ef | grep kdews | egrep -v 'ps|grep|startksrv|--agent' | awk '{print \$2;}'"
        // krthPID="ps -ef | grep krth | egrep -v 'ps|grep|startksrv|--agent' | awk '{print \$2;}'"
        // realtimeSERVER="cat /thomsonreuters/home/kplus/dist/adminbin/realtime.status | egrep 'RTMD|DRT'"
        // realtimeSERVERname="$realtimeSERVER | awk '{print \$1}'"
        // realtimeSERVERstatus="$realtimeSERVER | awk '{print \$2}'"
        // realtimeIDN="cat /thomsonreuters/home/kplus/dist/adminbin/realtime.status | grep IDN"
        // realtimeIDNstatus="$realtimeIDN | awk '{print \$2}'"
        return Ok(s);
    }

    async fn ssh_request(&self) -> Result<String, Box<dyn std::error::Error>> {
        let defaults: HashMap<&str, &str> =
            HashMap::from([
                ("user", "kplus"),
                ("ssh_key_path", "id_rsa"),
                ("port", "22"),
            ]);

        let username = match &self.username {
            Some(user) => user,
            None => defaults["user"],
        };

        let port = match &self.port {
            Some(port) => port,
            None => defaults["port"],
        };

        let private_key_path = match &self.private_key_path {
            Some(key) => key,
            None => defaults["ssh_key_path"],
        };

        let tcp = TcpStream::connect(format!("{}:{}", self.ip, port));
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp?);
        sess.handshake()?;
        sess.userauth_pubkey_file(&username, None, &Path::new(&private_key_path), None)?;

        // let mut channel = sess.channel_session()?;
        let s = Self::trigger_update(&self, &mut sess);
        return s;
    }
}

fn initiate_db(file_name: &str) {
    println!("Initiate DB");

    let connection = sqlite::open(file_name).unwrap();

    let query = "
        CREATE TABLE IF NOT EXISTS test_environment (ip TEXT, version INTEGER);
        INSERT INTO test_environment VALUES ('Bob', 69);
    ";
    connection.execute(query).unwrap();
}

#[tokio::main(flavor = "multi_thread", worker_threads = 100)]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut hosts_file = File::open("hosts.json")
        .or(File::open("hosts.json.example"))
        .expect("Expected 'hosts.json' or 'hosts.json.example', but cannot be found in current directory.");
    let mut hosts_file_buff = String::new();
    hosts_file.read_to_string(&mut hosts_file_buff)?;
    let mut processes: Vec<Process> =
        serde_json::from_str(&hosts_file_buff).expect("JSON was not well-formatted.");

    println!("{:?}", processes);

    initiate_db(DB_NAME);

    // tx, rx spawn
    let mut tokio_tx_handles = Vec::new();
    let (tx, rx) = channel();
    while let Some(process) = processes.pop() {
        let tx_clone: Sender<String> = tx.clone();
        tokio_tx_handles.push(tokio::spawn(async move {
            loop {
                std::thread::sleep(DELAY);
                if let Ok(output_data) = process.ssh_request().await {
                    let _ = tx_clone.send(output_data);
                };
            }
        }));
    }
    let rx_tokio_handle =
        tokio::spawn(async move {
            loop {
                println!("Fetched: {:?}", rx.recv().unwrap());
            }
        });

    let _ = rx_tokio_handle.await;
    Ok(())
}
