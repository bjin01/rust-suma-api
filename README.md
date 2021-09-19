# rust-suma-api
This program offers an rest API for Uyuni and or SUSE Manager.
You can call the url to get information about your system or trigger a patch job from Suse Manager.

curl -v http://<your-suma-hostname>/getid?hostname=clienthost.mydomain.com

The above command would query the system details from SUSE Manager and get displayed in HTML Code.

