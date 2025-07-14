# Minimal Terraform deployment for ICN nodes
provider "docker" {}

resource "docker_image" "icn_node" {
  name = "icn-node:latest"
  build {
    context    = "../../"
    dockerfile = "icn-devnet/Dockerfile"
  }
}

resource "docker_container" "node" {
  count  = var.node_count
  name   = "icn-node-${count.index}"
  image  = docker_image.icn_node.name
  env = [
    "ICN_NODE_NAME=Terraform-${count.index}",
    "ICN_ENABLE_P2P=true"
  ]
  ports {
    internal = 7845
    external = 6000 + count.index
  }
}

variable "node_count" {
  default = 3
}
