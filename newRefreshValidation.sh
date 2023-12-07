#!/bin/bash

. /home/micloud/.bash_profile
#. /thomsonreuters/home/kplus/.bash_profile

#declaring variables related to directories
TEMP=/home/micloud/scripts/refreshZones/tmp
WORKDIR=/home/micloud/scripts/refreshZones
DESTINATION_DIR=/home/micloud/portal/csportal/app/static/resources/
PORTAL_BASE_DIR=/home/micloud/portal/csportal
PORTAL_RESOURCES_DIR=/home/micloud/portal/csportal/app/static/resources
DESTINATION_DIR2=/nfs/vol5/support/CSAdmin/csportal_data/
#DESTINATION_DIR=/home/micloud/scripts/refreshZones/resources
DESTINATION_DIRS="/home/micloud/flask/application/static/resources/
/home/micloud/test/app/static/resources/"

cd $WORKDIR

#declaring variables for general utility

export VALIDATION_ZONES_LIST=$PORTAL_BASE_DIR/new_validation.txt
export SSHPASS="Kondor_123"
export PATH=$PATH:/usr/local/bin/
export ZODCLIENT="/home/micloud/scripts/broadcastZodclient.sh"
export PYTHONBIN="/home/micloud/.pyenv/versions/3.10.5/bin/python"

#variables for commands passed to environments
export GET_IP="/sbin/ifconfig -a | grep "inet" | grep 10. | awk '{print \$2}' | sed -e 's/addr://g' | egrep '^10\.'"
export GET_KPLUS_VER="cat dist/VERSION.kplus | awk '{print \$4}'"
GET_KPLUSTP_VER="if [ -e /kenvng/home/kplustp ]; then cat /kenvng/home/kplustp/dist/VERSION.ktpplus | sed 's/ //g' ; else echo K+TP_NO ; fi"
export GET_OS="if [ -e /etc/redhat-release ] ; then echo Linux ; else echo Solaris ; fi"
export GET_OS_VERSION="if [ -e /etc/release ]; then uname -r; else cat /etc/redhat-release | sed 's/ /_/g' ; fi"
export GET_DB="if grep DBMS_TYPE /kenvng/home/kplus/dist/entities/Standalone/config/kplus.params > /dev/null ; then
                grep DBMS_TYPE /kenvng/home/kplus/dist/entities/Standalone/config/kplus.params | grep -v ENTITY | awk '{print \$2}' ; else echo unknown ; fi"
export KPLUSTP_DIR="/kenvng/home/kplustp"
export KGR_DIR="/kenvng/home/kgr"
GET_KGR_VER="if [ -e $KGR_DIR ]; then cat /kenvng/home/kgr/VERSION.fusionrisk | sed 's/ //g' ; else echo KGR_NO ; fi"
GET_ELS_VERSION="if [ -e /kenvng/els/VERSION.els ]; then nawk 1 /kenvng/els/VERSION.els | sed 's/ /_/g' ; else echo unknown; fi"
GET_ELS_EXPIRATION="if [ -e /etc/release ]; then ggrep -m1 APPLICATION /kenvng/els/license.def | ggrep -o '[0-9]*/[0-9]*/[0-9]*' ; else grep -m1 APPLICATION /kenvng/els/license.def | grep -o '[0-9]*/[0-9]*/[0-9]*' ; fi"
GET_RVD_VERSION="if [ -e /kenvng/rvd/release_notes/ ]; then ls -l /kenvng/rvd/release_notes/ | grep -v html | grep TIB | awk '{print \$9}' | sed 's/_license.pdf//g' ; else echo unknown ; fi"
GET_PYTHON_VER="python -V 2>&1 | sed 's/ //g'"
GET_JAVA_VERSION="/kenvng/java/jdk/bin/java -version 2>&1 | grep version | sed 's/ //g' | sed 's/java//g'"
GET_RTMD_VERSION="cat /kenvng/home/kplus/rtmd/rtmd.version | grep RealTime | sed 's/[^0-9.]*//g'"


#variables for database queries
GET_DATE_LOCAL='date +%d/%m/%Y'
GET_DB_DATE="if [ -e /nfs/vol5/csrepository/getDBDate.sh ]; then /nfs/vol5/csrepository/getDBDate.sh ; else echo missing_repository ; fi"

GET_DATASERVER="grep DBMS_TYPE /kenvng/home/kplus/dist/entities/Standalone/config/kplus.params | grep -v ENTITY | awk '{print \$2}'"
GET_DBSERVER_IP="nslookup \$(grep DATASERVER_HOST /kenvng/home/kplus/dist/entities/Standalone/config/kplus.params | awk '{print \$2}') |
    grep -A2 Non-authoritative | grep Address | awk '{print \$2}'"
GET_DB_VERSION="if [ -e /nfs/vol5/csrepository/getDBVersion.sh ]; then /nfs/vol5/csrepository/getDBVersion.sh ; else echo missing_repository ; fi"

#boolean for determining if the zone is down or online
PROCESS_ZONE="FALSE"

#variables for the process statuses
kmsPID="ps -ef | grep kms | egrep -v 'ps|grep|startksrv|--agent' | awk '{print \$2;}'"
klwsPID="ps -ef | grep klws | egrep -v 'ps|grep|startksrv|--agent' | awk '{print \$2;}'"
kdewsPID="ps -ef | grep kdews | egrep -v 'ps|grep|startksrv|--agent' | awk '{print \$2;}'"
krthPID="ps -ef | grep krth | egrep -v 'ps|grep|startksrv|--agent' | awk '{print \$2;}'"
realtimeSERVER="cat /thomsonreuters/home/kplus/dist/adminbin/realtime.status | egrep 'RTMD|DRT'"
realtimeSERVERname="$realtimeSERVER | awk '{print \$1}'"
realtimeSERVERstatus="$realtimeSERVER | awk '{print \$2}'"
realtimeIDN="cat /thomsonreuters/home/kplus/dist/adminbin/realtime.status | grep IDN"
realtimeIDNstatus="$realtimeIDN | awk '{print \$2}'"

#gathering validation zones function
getValidationZones() {

cd $WORKDIR

$ZODCLIENT "nextgen_cs_support nextgen_cs_support_gdn" "--action search --ls --parseable --format 'Booted,FQDN' --desc validation" | sed 's/.misys.global.ad//g' > $PORTAL_BASE_DIR/tmp_new_validation.txt

#sorting the final validation list
cd $PORTAL_BASE_DIR
cat tmp_new_validation.txt | sort -k2 > new_validation.txt
rm tmp_new_validation.txt

}

getZonesWithModules() {

$ZODCLIENT "nextgen_cs_support nextgen_cs_support_gdn" "--action search --ls --parseable --format 'Booted,FQDN,Usage' --desc keys" | sed 's/.misys.global.ad//g' > $PORTAL_RESOURCES_DIR/zones_with_keys.txt
$PYTHONBIN $PORTAL_BASE_DIR/app/syncModules.py
}

#broadcast command function
broadcastCommands() {

for ZONE in $(cat $VALIDATION_ZONES_LIST);
do
    # cd $WORKDIR
    case $ZONE in
        yes)

            PROCESS_ZONE="TRUE"
            ;;

        no)

            PROCESS_ZONE="FALSE"
            ;;

        #the default case, the base starting point for gathering data for our ONLINE internal zones
        *)

            echo "zone_host::$ZONE"
            if [ $PROCESS_ZONE = "TRUE" ]
            then

                sshpass -e ssh -o StrictHostKeyChecking=no kplus@$ZONE "echo -n zone_ip:: ; $GET_IP ;
                                            echo -n zone_kplusver:: ; $GET_KPLUS_VER ;
                                            echo -n zone_kplustp:: ; if [ -e $KPLUSTP_DIR ]; then echo K+TP_YES ; else echo K+TP_NO ; fi ;
                                            echo -n zone_kplustp_ver:: ; $GET_KPLUSTP_VER ;
                                            echo -n zone_kgr:: ; if [ -e $KGR_DIR ]; then echo KGR_YES ; else echo KGR_NO ; fi ;
                                            echo -n zone_kgr_ver:: ; $GET_KGR_VER ;
                                            echo -n zone_os:: ; $GET_OS ;
                                            echo -n zone_os_ver:: ; $GET_OS_VERSION ;
                                            echo -n zone_dataserver:: ; $GET_DB ;
                                            if [[ \$(eval $GET_DATASERVER) = 'SQL_SERVER' ]] ; then echo -n zone_dbserver_ip:: ; $GET_DBSERVER_IP ; fi ;
                                            echo -n zone_els_ver:: ; $GET_ELS_VERSION ;
                                            echo -n zone_els_exp:: ; $GET_ELS_EXPIRATION ;
                                            echo -n zone_rvd_ver:: ; $GET_RVD_VERSION ;
                                            echo -n zone_python_ver:: ; $GET_PYTHON_VER ;
                                            echo -n zone_java_ver:: ; $GET_JAVA_VERSION"
                sshpass -e ssh -T kplus@$ZONE "echo -n zone_dbdate:: ; $GET_DB_DATE ;
                                            echo -n zone_dbver:: ; $GET_DB_VERSION"
#this next part is not pretty, it is necesarry
                sshpass -e ssh kplus@$ZONE "echo -n zone_kms:: ; if [[ -n \$(eval $kmsPID) ]]; then echo KMS_ON ; else echo KMS_OFF ; fi ;
			echo -n zone_klws:: ; if [[ -n \$(eval $klwsPID) ]]; then echo KLWS_ON ; else echo KLWS_OFF ; fi ;
			echo -n zone_kdews:: ; if [[ -n \$(eval $kdewsPID) ]]; then echo KDEWS_ON ; else echo KDEWS_OFF ; fi ;
			echo -n zone_krth:: ; if [[ -n \$(eval $krthPID) ]]; then echo KRTH_ON ; else echo KRTH_OFF ; fi ;
			echo -n zone_realtime:: ; if [[ -n \$(eval $realtimeSERVER) ]];
				then if [ $( printf '$('"$realtimeSERVERstatus"')' ) = "Up" ] ;
					then echo $( printf '$('"$realtimeSERVERname"')_ON' ) ; else echo $( printf '$('"$realtimeSERVERname"')_OFF' ) ; fi
				else echo REALTIME_NO ; fi ;
            if [[ \$(eval $realtimeSERVERname) = 'RTMD' ]] ; then echo -n zone_rtmd_ver:: ; $GET_RTMD_VERSION ; fi ;
			echo -n zone_idn:: ; if [[ -n \$(eval $realtimeIDN) ]]; then if [ $( printf '$('"$realtimeIDNstatus"')' ) = "Up" ] ; then echo IDN_ON ; else echo IDN_OFF ; fi else echo IDN_OFF ; fi "

                echo zone_status::UP
            else
            #this is the beggining of the branching point for OFFLINE zones, changing based on the zone domain
                case $ZONE in
                    #supvkenv zones
                    supvkenv*)

                        ZONENUMBER=`echo $ZONE | sed 's/.*\(...\)/\1/' | sed 's/^0*//'`
                        if (( $ZONENUMBER >= 100 && $ZONENUMBER <= 150 )) ; then
                            ZOD_DOMAIN="support_smt" ;
                            ZONE_OS="Solaris"
                        else
                            ZOD_DOMAIN="nextgen_cs_support" ;
                            ZONE_OS="Linux"
                        fi

                        ZONENAME=`echo $ZONE | sed 's/supvkenv/kzone/g'`
                        ;;

                    #csvnext zones
                    csvnext*)

                        ZONENUMBER=`echo $ZONE | sed 's/.*\(...\)/\1/' | sed 's/^0*//'`
                        if (( $ZONENUMBER >= 111 && $ZONENUMBER <= 200 )) ; then
                            ZOD_DOMAIN="nextgen_cs_support" ;
                        else
                            ZOD_DOMAIN="nextgen_cs_support_gdn" ;
                        fi

                        ZONENAME=`echo $ZONE | sed 's/csvnext/kzone/g'`
                        ZONE_OS="Linux"
                        ;;
                esac

                echo zone_ip::1.1.1.1 ;
                echo -n zone_kplusver:: ; $ZODCLIENT "$ZOD_DOMAIN" "--zonename $ZONENAME --action search --ls --parseable --format 'Disk'" | sed 's/,/\n/g' | grep home.kplus.3 | awk -F. '{print $3}' | sed 's/./&./1;s/./&./3;s/./&./5' ;
                echo zone_os::$ZONE_OS ; echo zone_dataserver::DB ; echo `zone_dbdate::$GET_DATE_LOCAL` ; echo zone_kplustp::K+TP_? ; echo zone_kgr::KGR_? ;
                echo zone_kms::unknown; echo zone_klws::unknown; echo zone_kdews::unknown; echo zone_krth::unknown; echo zone_realtime::unknown; echo zone_idn::unknown; echo zone_status::DOWN
            fi
            ;;
    esac
done
#echo "script done"
}

#==============MAIN PROGRAM===============================================================
echo "Starting the harvesting of validation zones..."

#establishing the validation counter and creating it in case it doesn't exist
if [ -e $PORTAL_RESOURCES_DIR/validation.counter ]; then
counter=$(cat $PORTAL_RESOURCES_DIR/validation.counter)
else
    counter=0
    echo -n $counter > $PORTAL_RESOURCES_DIR/validation.counter
fi

#when counter is 0 we perform the full refresh, otherwise we are doing a series of fast refreshes
if (( $counter == 0 )); then
    echo "Doing full refresh"
    getValidationZones
    echo -e "\n new_validation.txt should be prepared for the mass broadcast"
    counter=$(($counter+1))
elif (( $counter == 10 )); then
    echo "Doing last fast refresh"
    counter=0
else
    echo "Doing fast refresh"
    counter=$(($counter+1))
fi

echo -n $counter > $PORTAL_RESOURCES_DIR/validation.counter

#zones with modules initiative
getZonesWithModules

echo -e "\n Starting the mass broadcast..."

#saving unparsed results and processing the file for the final valid input
broadcastCommands > $PORTAL_BASE_DIR/new_result.txt

echo -e "\nAdjusting the final output for Flask..."

cat $PORTAL_BASE_DIR/new_result.txt | sed -e ':a;N;$!ba;s/\n/ /g' | sed -e 's/UP /UP\n/g' | sed -e 's/DOWN /DOWN\n/g' > $PORTAL_BASE_DIR/new_valid_input.txt

#sorting by different colmun option
cat $PORTAL_BASE_DIR/new_valid_input.txt | sort -k3 -r > $PORTAL_RESOURCES_DIR/valid_input.txt

echo -e "\nScript done. Check the valid_input.txt file and portal"
exit 1
