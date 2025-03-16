gnome.compile_resources('demo', 'demo.gresource.xml', gresource_bundle: true)

configure_file(input: 'config.rs.in', output: 'config.rs', configuration: conf)

# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.current_build_dir() / 'config.rs',
  meson.current_source_dir() / 'config.rs',
  check: true,
)

cargo_demo_build = custom_target(
  'cargo-build-demo',
  build_by_default: true,
  build_always_stale: true,
  output: 'pfs-demo',
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    'env',
    cargo_env,
    cargo_bin,
    'build',
    cargo_opt,
    '&&',
    'cp',
    meson.project_build_root() / 'src' / rust_target / 'pfs-demo',
    '@OUTPUT@',
  ],
)
