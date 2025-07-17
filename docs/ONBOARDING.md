# ICN Core Onboarding Guide

Welcome to the InterCooperative Network (ICN) Core project! This guide helps you get started with ICN development and deployment.

## Quick Start

**New to ICN?** Start here for the fastest path to a working development environment:

📚 **[Quick Start Guide →](QUICKSTART.md)**  
Get running in under 10 minutes with minimal setup

## Developer Resources

### Getting Started
- 🏗️ **[Developer Guide](DEVELOPER_GUIDE.md)** - Comprehensive development documentation
- 🧱 **[Architecture Overview](ARCHITECTURE.md)** - System design and component relationships  
- 📖 **[API Reference](API.md)** - Complete API documentation
- 🔧 **[Contributor Setup](CONTRIBUTOR_SETUP.md)** - Detailed development environment setup

### Advanced Topics
- 🏛️ **[Governance Framework](governance-framework.md)** - Understanding ICN governance
- 💰 **[Economics Models](economics-models.md)** - Mana system and resource allocation
- 🔮 **[CCL Language Reference](CCL_LANGUAGE_REFERENCE.md)** - Cooperative Contract Language
- 🔐 **[Security Guide](PRODUCTION_SECURITY_GUIDE.md)** - Security best practices

## Deployment & Operations

### Production Deployment
- 🚀 **[Deployment Guide →](deployment-guide.md)**  
  Complete guide for production deployments
- 📊 **[Monitoring](monitoring.md)** - Observability and health checks
- 🔧 **[Troubleshooting](TROUBLESHOOTING.md)** - Common issues and solutions

### Development Networks
- 🌐 **[Multi-Node Setup](MULTI_NODE_GUIDE.md)** - Local federation testing
- 🐳 **[Devnet Guide](../icn-devnet/README.md)** - Containerized development network

## Federation & Governance

### Cooperative Operations
- 🤝 **[Federation Trust](FEDERATION_TRUST.md)** - Trust relationships between cooperatives
- ⚖️ **[Governance Onboarding](governance_onboarding.md)** - Participatory governance setup
- 📋 **[Governance Patterns](governance-pattern-library.md)** - Common governance templates

### Interactive Setup
Use the CLI wizards for guided setup:

```bash
# Developer onboarding with test data
cargo run -p icn-cli -- wizard init-dev

# Federation setup wizard
cargo run -p icn-cli -- wizard onboard-federation
```

## Reference Documentation

### Technical Specifications  
- 📚 **[Feature Overview](ICN_FEATURE_OVERVIEW.md)** - Complete feature matrix
- 🔄 **[Event Sourcing](EVENT_SOURCING.md)** - Event-driven architecture details
- 🗃️ **[Glossary](GLOSSARY.md)** - ICN terminology and concepts

### Development Workflows
- 🧪 **[Testing Guide](large_scale_testing.md)** - Testing strategies and tools
- 🔁 **[CI/CD Workflow](../.github/workflows/)** - Automated testing and deployment

## Community & Support

### Getting Help
- 💬 **Issues & Discussions** - [GitHub Issues](https://github.com/InterCooperative-Network/icn-core/issues)
- 📖 **Documentation** - This repository's `/docs` folder
- 🤔 **Questions** - Use the [Question template](.github/ISSUE_TEMPLATE/question.md)

### Contributing
- 🏗️ **[Contributing Guide](../CONTRIBUTING.md)** - How to contribute to ICN
- 📋 **[Code of Conduct](../CODE_OF_CONDUCT.md)** - Community guidelines
- 🎯 **[Development Roadmap](COOPERATIVE_ROADMAP.md)** - Project direction

---

## Need Help?

- **Quick Questions**: Check the [Troubleshooting Guide](TROUBLESHOOTING.md)
- **Feature Requests**: Use our [Feature Request Template](../.github/ISSUE_TEMPLATE/feature_request.yml)
- **Bug Reports**: Use our [Bug Report Template](../.github/ISSUE_TEMPLATE/bug_report.yml)
- **Governance Proposals**: Use our [RFC Template](../.github/ISSUE_TEMPLATE/rfc.md)

**Ready to start?** Jump to the **[Quick Start Guide](QUICKSTART.md)** to get your development environment running! 