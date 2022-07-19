==Setup configuration defaults==

* Overview

Configuration and data files are stored under "%APPDATA%\rclip" folder
- You can paste the location in the Windows explorer location bar
- You can also switch  to the configuration folder from the command-line

* Setup

1. Please make sure that you have OpenSSL installed. See https://winget.run/pkg/ShiningLight/OpenSSL

2. Generate SSL certificates and copy other files by running the "configure.bat" script

3. Copy binaries and the public key to other machines
  - The public key "der-cert-pub.der" will be in one of the folders describe in the first section of this document.
  - The public key needs to be at the same location on other machines
  - The public key on other machines needs to be exactly the same as the one on the server!


