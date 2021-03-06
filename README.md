# tokio-agnostic-uds

This project takes [this repo](https://github.com/Azure/tokio-uds-windows) and updates it from using tokio 0.1 to tokio 0.2. Importantly, the upgrade is accomplished without the need of tokio-compat. Additionally, this repo automatically switches between the aforementioned repo and tokio's UDS implementation depending on the build target. The use of rust's zero-cost abstractions is used to ensure there's no loss in performance.

Check the examples directory for an example of using the software

Supports Windows 10 + Linux + MacOS

## Windows support for Unix domain sockets
Support for Unix domain sockets was introduced in Windows 10. It became generally available in version
1809 (aka the October 2018 Update), and in Windows Server 1809/2019.
