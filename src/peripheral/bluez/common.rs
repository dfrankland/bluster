use dbus::tree::{MTFn, Tree as DbusTree};
use dbus_tokio::tree::ATree;

pub type Tree = DbusTree<MTFn<ATree<()>>, ATree<()>>;
