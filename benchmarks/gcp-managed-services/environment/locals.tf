locals {
  prefix = "axb-${var.run_id}"
  labels = {
    purpose = "axiom-managed-bench"
    run_id  = var.run_id
  }

  receiver_image = "${var.registry}/receiver:${var.image_tag}"
  tape_image     = "${var.registry}/tape:${var.image_tag}"
  defer_image    = "${var.registry}/defer:${var.image_tag}"
  relay_image    = "${var.registry}/relay:${var.image_tag}"
  client_image   = "${var.registry}/client:${var.image_tag}"
}
