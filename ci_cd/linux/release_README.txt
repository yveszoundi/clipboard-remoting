==Setup configuration defaults==

* Overview

It can be cumbersome to always provide configuration parameters over and over again.
rclip supports configuration files in locations that follow operating system conventions:
- Under Linux or BSD, the "$XDG_CONFIG_HOME" and "$XDG_DATA_HOME" folders
- Under Windows, the "%APPDATA%" folder
- Under MacOS, the "$HOME/Library/Application\ Support" folder

* Setup

1. Please make sure that you have OpenSSL installed
  - Under windows see https://winget.run/pkg/ShiningLight/OpenSSL
  - Under Linux, usually OpenSSL is already installed

2. Install the default configuration files from the command line
  - Under Linux: ./configure.sh
  - Under Windows: configure.bat

3. Copy binaries and the public key to other machines
  - The public key "der-cert-pub.der" will be in one of the folders describe in the first section of this document.
  - The public key needs to be at the same location on other machines
  - The public key on other machines needs to be exactly the same as the one on the server!


