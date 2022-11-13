# rpmmake

rpmmake is an experimental rewrite of the [mock](https://rpm-software-management.github.io/mock/) tool in Rust.

## Why?
Mock is a great tool, but it has several issues:
- Mock takes around 2 minutes to even simply initialize a chroot, prepare the whole environment and install build dependencies to even start building a package.
- Mock is very complex and hard to extend. There are many hooks and plugins, but they are not well documented and it is hard to understand how they work.
- Mock's jinja2 template configs are hard to read and understand.
- Mock is written in Python, which is not the fastest language.

## What is rpmmake?

rpmmake aims to be a simple and fast tool for building reproducible RPM packages. It relies upon bubblewrap to create a chroot environment and aims to quickly build packages without any unnecessary steps.

Currently, rpmmake is in a very early stage of development and is not ready for production use.

rpmmake only supports building RPMs using `rpmbuild` for now. A pure-rust implementation of rpmbuild is planned.

## Known issues

- rootless overlayfs is broken due to permission issues. however if you mount the overlayfs manually, it works fine.