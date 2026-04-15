use pyo3::prelude::*;

mod config;
mod container;
pub mod errors;
pub mod file;
mod http;
mod kubectl;
mod network;
pub mod runtime;
mod service;
mod shell;
mod ssh;
mod validate;

/// The native core module for cutip-blocks.
/// Exposed to Python as `cutip_blocks._core`.
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Errors
    errors::register(m)?;
    // Shell
    m.add_function(wrap_pyfunction!(shell::run, m)?)?;
    // SSH
    m.add_class::<ssh::ExecResult>()?;
    m.add_class::<ssh::SSHSession>()?;
    m.add_function(wrap_pyfunction!(ssh::ssh_connect, m)?)?;
    // kubectl
    m.add_class::<kubectl::KubectlSession>()?;
    m.add_function(wrap_pyfunction!(kubectl::kubectl_connect, m)?)?;
    // Container
    m.add_class::<container::ContainerRuntime>()?;
    m.add_class::<container::ContainerExecResult>()?;
    m.add_function(wrap_pyfunction!(container::container_connect, m)?)?;
    // Network
    m.add_function(wrap_pyfunction!(network::create, m)?)?;
    m.add_function(wrap_pyfunction!(network::remove, m)?)?;
    m.add_function(wrap_pyfunction!(network::exists, m)?)?;
    // File
    m.add_function(wrap_pyfunction!(file::copy, m)?)?;
    m.add_function(wrap_pyfunction!(file::copy_tree, m)?)?;
    m.add_function(wrap_pyfunction!(file::read_yaml, m)?)?;
    m.add_function(wrap_pyfunction!(file::write_yaml, m)?)?;
    m.add_function(wrap_pyfunction!(file::read_json, m)?)?;
    m.add_function(wrap_pyfunction!(file::write_json, m)?)?;
    m.add_function(wrap_pyfunction!(file::replace, m)?)?;
    m.add_function(wrap_pyfunction!(file::mkdir, m)?)?;
    m.add_function(wrap_pyfunction!(file::is_empty, m)?)?;
    // HTTP
    m.add_class::<http::HttpResponse>()?;
    m.add_function(wrap_pyfunction!(http::get, m)?)?;
    m.add_function(wrap_pyfunction!(http::post, m)?)?;
    m.add_function(wrap_pyfunction!(http::put, m)?)?;
    m.add_function(wrap_pyfunction!(http::delete, m)?)?;
    // Service
    m.add_function(wrap_pyfunction!(service::poll_until_ready, m)?)?;
    m.add_function(wrap_pyfunction!(service::wait_for_exit, m)?)?;
    // Validate
    m.add_function(wrap_pyfunction!(validate::path_exists, m)?)?;
    m.add_function(wrap_pyfunction!(validate::env_var_set, m)?)?;
    m.add_function(wrap_pyfunction!(validate::ip_valid, m)?)?;
    // Config
    m.add_function(wrap_pyfunction!(config::render_template, m)?)?;
    m.add_function(wrap_pyfunction!(config::substitute_vars, m)?)?;
    Ok(())
}
