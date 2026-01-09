# Security Policy

## Supported Versions

The following versions of `one2html` are currently supported with security updates:

| Version | Supported |
| ------- | --------- |
| Latest release | ✅ |
| Older releases | ❌ |

## Reporting a Vulnerability

We take security issues seriously. If you discover a security vulnerability in `one2html`, please report it responsibly.

### How to Report

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via one of the following methods:

1. **GitHub Private Vulnerability Reporting**:  
   https://github.com/msiemens/one2html/security/advisories/new

2. **Email**: Contact the maintainer directly at the email address listed on the GitHub profile:  
   https://github.com/msiemens

### What to Include

Please include as much of the following information as possible:

- A description of the vulnerability
- Steps to reproduce the issue (ideally with a minimal proof-of-concept)
- The input file(s) used (or a sanitized/minimized reproduction, if the original contains sensitive data)
- Affected versions / commit hash
- Observed and expected behavior
- Any suggested fixes or mitigations (if available)

### What to Expect

- **Acknowledgment**: You should receive an acknowledgment within **7 days** of your report.
- **Updates**: We will keep you informed about the progress of addressing the vulnerability.
- **Resolution**: There is no fixed timeline for resolution; it will depend on severity, complexity, and maintainer availability.
- **Credit**: We are happy to credit security researchers who report valid vulnerabilities (unless you prefer to remain anonymous).

## Security Considerations

`one2html` converts OneNote content into HTML and should be treated as a tool that processes potentially untrusted input files.

### HTML Output Safety

The generated HTML may include user-controlled content from the source OneNote file. If you publish or render the output in a browser or embed it into another page:

- Treat the output as **untrusted** unless you control the input.
- Consider applying additional sanitization (especially if you will serve the output on the web or embed it into an existing site).
- Be cautious of active content (e.g., links) and any downstream processing that might introduce script execution.

### Resource Exhaustion / Denial of Service

Reports about excessive memory or CPU usage are welcome **when they can be demonstrated with realistically sized inputs**.

Issues that require **extremely large input files** to trigger memory exhaustion (or similar resource exhaustion) are generally considered **out of scope**, unless the same behavior can be reproduced with regular-sized files or typical real-world OneNote content.

## Dependency Security

We aim to keep dependencies up to date and respond to relevant upstream security advisories (for example via RustSec).