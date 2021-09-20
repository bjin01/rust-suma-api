# rest api for suma in rust
This program offers an rest API for Uyuni and or SUSE Manager.
You could make HTTP GET requests to 
* get basic information about a given system hostname.
* schedule a patch job on SUSE Manager for the given host.

## Benefits:
* allows client/minion side patch job scheduling; 
* convenient to use salt state cmd.run or ansible playbook to trigger the schedule job but without loosing job history on SUSE Manager;

## Downad and Install
download the binary to your local host.
```
cd /usr/local/bin
wget https://github.com/bjin01/rust-suma-api/raw/master/uysurest
chmod +x uysurest
cd
```
Create the config file in yaml format and provide login credentials to SUSE Manager, provide the tls certificate and private key file names and the port number for the rest api program to use.
```
---
hostname: bjsuma.bo2go.home
user_name: bjin
password: suse1234
certificate: /home/bjin/tmp/mycert.pem
tls_key: /home/bjin//tmp/mykey.pem
restapi_port: 8888
```
Start the program:
```
uysurest --config /home/bjin/config.yaml
```

## Sample HTTP GET requests:
### getinfo
Below GET request would query the system details from SUSE Manager and get some parameters displayed in HTML Code.
```
curl -v https://your-suma-hostname:8888/getinfo?hostname=caasp01.bo2go.home
```

__Sample output:__

```
<p>minion_id: caasp01.bo2go.home</p><p>machine_id: 235294fd17e14b699bc18fb0e989c3bb</p><p>base_entitlement: salt_entitled</p><p>virtualization: KVM/QEMU</p><p>contact_method: default</p>
```
### patch
Below GET request would schedule a patch job to the host. All relevant patches will be applied.

```
curl -v -k https://your-suma-hostname:8888/patch?hostname=caasp01.bo2go.home
```

__Sample output:__
```
Jobid: 11821
```

