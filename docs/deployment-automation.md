# Deployment Automation

This guide outlines example Terraform and Ansible configurations for provisioning ICN nodes.

## Terraform

The `deploy/terraform` directory contains a minimal configuration using the Docker provider. Adjust `node_count` to launch multiple containers:

```bash
cd deploy/terraform
terraform init
terraform apply -var="node_count=5"
```

## Ansible

The `deploy/ansible` playbook assumes hosts grouped under `icn` in your inventory. It installs Docker and runs the ICN node image:

```bash
ansible-playbook -i inventory deploy/ansible/playbook.yml -e node_count=5
```

These samples are intentionally lightweight and should be adapted for production environments.
