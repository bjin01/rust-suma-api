# rest api for suma in rust with HTTP Basic Authentication
This program offers a simple rest API for Uyuni and or SUSE Manager.
You could make HTTP GET requests to 
* get basic information about a given system hostname.
* schedule a patch job on SUSE Manager for the given host.

## Benefits:
* allows client/minion side patch job scheduling; 
* convenient to use salt state cmd.run or ansible playbook to trigger the schedule job but without loosing job history on SUSE Manager;

## Downad and Install
Download the binary to your local host.
```
wget https://github.com/bjin01/rust-suma-api/files/7231314/uysurest.tar.gz
sudo tar -xvzf uysurest.tar.gz -C /usr/local/bin/
chmod +x /usr/local/bin/uysurest
```
Create TLS Certificate for your rest api host: 
 
If this program is running on SUSE Manager host itself you could re-use the existing server certficate file and key file located on 
```
/etc/apache2/ssl.crt/server.crt
/etc/apache2/ssl.key/server.key
```
You will need the tls certificate for the host where this program will run and also the key file. The path to this files will be needed in the next step.
Of course you could use official signed server certificate and your key file.
For example:

```
cd /tmp
openssl req -x509 -newkey rsa:4096 -nodes -keyout mykey.pem -out mycert.pem -days 365 -subj '/CN=localhost'
```

Create the config file in yaml format and provide login credentials to SUSE Manager, provide the tls certificate and private key file names and the port number for the rest api program to use.
```
---
hostname: bjsuma.bo2go.home
user_name: bjin
password: suse1234
certificate: /tmp/mycert.pem
tls_key: /tmp/mykey.pem
restapi_port: 8888
http_basic_auth_user: suma
http_basic_auth_password: restapi
```
Start the program:
```
uysurest --config /home/bjin/config.yaml
```

## Sample HTTP GET requests:
### getinfo
Below GET request would query the system details from SUSE Manager and get some parameters displayed in HTML Code.
```

curl -v -k -u suma:restapi https://127.0.0.1:8888/getinfo?hostname=caasp01.bo2go.home
```

__Sample output:__

```
<p>minion_id: caasp01.bo2go.home</p><p>machine_id: 235294fd17e14b699bc18fb0e989c3bb</p><p>base_entitlement: salt_entitled</p><p>virtualization: KVM/QEMU</p><p>contact_method: default</p>
```
### patch
Below GET request would schedule a patch job to the host. All relevant patches will be applied.

```
curl -v -k -u suma:restapi https://your-suma-hostname:8888/patch?hostname=caasp01.bo2go.home
```

__Sample output:__
```
Jobid: 11821
```

