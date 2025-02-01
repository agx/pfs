desktopconf = configuration_data()
desktopconf.set('DEMO', demo)
desktop_file = 'mobi.phosh.FileSelectorDemo.desktop'
merged = i18n.merge_file(
  input: configure_file(
    input: desktop_file + '.in.in',
    output: desktop_file + '.in',
    configuration: desktopconf,
  ),
  output: desktop_file,
  po_dir: '../po',
  install: get_option('examples'),
  install_dir: get_option('datadir') / 'applications',
  type: 'desktop',
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils, args: [merged])
endif

appstream_file = i18n.merge_file(
  input: 'mobi.phosh.FileSelectorDemo.metainfo.xml.in',
  output: 'mobi.phosh.FileSelectorDemo.metainfo.xml',
  po_dir: '../po',
  install: get_option('examples'),
  install_dir: get_option('datadir') / 'metainfo',
)

appstreamcli = find_program('appstreamcli', required: false, disabler: true)
test(
  'Validate appstream file',
  appstreamcli,
  args: ['validate', '--no-net', '--explain', appstream_file],
)

install_data(
  'mobi.phosh.FileSelector.gschema.xml',
  install_dir: get_option('datadir') / 'glib-2.0' / 'schemas',
)

compile_schemas = find_program('glib-compile-schemas', required: false)
if compile_schemas.found()
  test(
    'Validate schema file',
    compile_schemas,
    args: ['--strict', '--dry-run', meson.current_source_dir()],
  )
endif
compiled = gnome.compile_schemas(build_by_default: true)

subdir('icons')
