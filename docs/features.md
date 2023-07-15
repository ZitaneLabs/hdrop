# Self-Host Ready File Hoster

This repository contains a self-hosted file hosting solution that offers a wide range of features for secure and convenient file management.

## Features
- **Configurability**: Modify specific server options and properties without the need to touch the code, Hdrop provides a list of configurable options for tailoring the file hoster to your specific needs.
- **Minimal Data Collection**: The server does not require user accounts, does not store passwords, and avoids unnecessary metadata on the servers.
- **Multiple Storage Options**: Choose between local storage hosting or seamless integration with S3-compatible storage providers.
- **End-to-End Encryption (E2EE)**: Ensure security with encryption applied both at rest and during transit, protecting your files end-to-end.
- **Web Crypto API Integration**: Utilize the Web Crypto API to leverage cryptographic primitives for enhanced security and data integrity.
- **Automated Secure Password/Key Generation**: Generate strong and secure passwords/keys automatically, eliminating the need for manual key management.
- **Full Backend and UI Frontend**: Benefit from a complete solution that includes both a rust backend for file management and a nextjs UI frontend for easy interaction.
- **Metrics with Prometheus**: Monitor and analyze system performance and usage statistics using Prometheus.
- **Extensive Logging/Tracing**: Gain valuable insights into system operations through comprehensive logging and tracing capabilities, facilitating troubleshooting and auditing.
- **Protection Against Unauthorized Downloads**: Implemented safeguards to prevent unauthorized downloads by requiring the correct password.
- **Direct Accessibility of Content**: Enables users to directly access and view downloaded content, such as text files, images, and videos, directly in their browsers.

Refer to the repository documentation and guides for detailed instructions on deploying and configuring Hdrop.
