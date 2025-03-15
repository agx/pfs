if not get_option('examples')
  subdir_done()
endif

executable(
  'pfs-c-demo',
  ['example.c'],
  include_directories: pfs_inc,
  link_with: cargo_static_lib_build,
  dependencies: [gtk_dep, libadwaita_dep],
)

subdir('demo')
