terraform {
  required_version = ">= 1.0.0"
}

variable "node_count" {
  type    = number
  default = 3
}

provider "docker" {}

resource "docker_network" "icn" {
  name = "icn-federation"
}

resource "docker_image" "icn" {
  name         = "intercooperative/icn-node:latest"
  keep_locally = true
}

resource "docker_container" "icn_node" {
  count  = var.node_count
  name   = "icn-node-${count.index}"
  image  = docker_image.icn.name
  networks_advanced {
    name = docker_network.icn.name
  }
  ports {
    internal = 7845
    external = 5001 + count.index
  }
  restart = "always"
  command = ["icn-node", "--bootstrap-peers", ""]
}
