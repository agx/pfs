if not get_option('examples')
  subdir_done()
endif

foreach example : examples

  scalable_dir = 'hicolor' / 'scalable' / 'apps'
  install_data(
    scalable_dir / ('@0@.svg').format(example[0]),
    install_dir: get_option('datadir') / 'icons' / scalable_dir,
  )

  symbolic_dir = 'hicolor' / 'symbolic' / 'apps'
  install_data(
    symbolic_dir / ('@0@-symbolic.svg').format(example[0]),
    install_dir: get_option('datadir') / 'icons' / symbolic_dir,
  )

endforeach
