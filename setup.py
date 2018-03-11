#!/usr/bin/env python3
"""
Setup the library.
"""
import sys

from setuptools import setup

try:
    from setuptools_rust import Binding, RustExtension
except ImportError:
    import subprocess

    err = subprocess.call([sys.executable, '-m', 'pip', 'install', 'setuptools_rust'])
    if err:
        print('Please install setuptools-rust package')
        raise SystemError(err)
    else:
        try:
            from setuptools_rust import Binding, RustExtension
        except ImportError:
            raise SystemError('setuptools-rust package was not installed')

setup_requires = ['setuptools-rust>=0.8.4']
install_requires = []
tests_require = install_requires + []

setup(
    name='pygui',
    version='0.1.0',
    description='A simple python gui library written in rust',
    author='Juici',
    author_email='juicy66173@gmail.com',
    url='https://github.com/Juici/pygui',
    packages=['pygui'],
    classifiers=[
        'Development Status :: 3 - Alpha',
        'Intended Audience :: Developers',
        'Intended Audience :: Education',
        'License :: OSI Approved :: MIT License',
        'Natural Language :: English',
        'Operating System :: MacOS :: MacOS X',
        'Operating System :: Microsoft :: Windows',
        'Operating System :: POSIX',
        'Programming Language :: Python',
        'Programming Language :: Rust',
        'Topic :: Education',
        'Topic :: Software Development :: Libraries :: Python Modules',

    ],
    rust_extensions=[
        RustExtension('pygui._pygui', 'Cargo.toml', binding=Binding.PyO3, rust_version='>=1.26')
    ],
    install_requires=install_requires,
    tests_require=tests_require,
    setup_requires=setup_requires,
    include_package_data=True,
    zip_safe=False,
)
