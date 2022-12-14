Name: near-facsimile
Summary: Find similar or identical text files in a directory
Version: 1.0.2
Release: 1%{?dist}
License: ASL 2.0
URL: https://github.com/msuchane/near-facsimile
Group: Applications/Text
Source0: https://static.crates.io/crates/%{name}/%{name}-%{version}.crate
#Source1: https://github.com/msuchane/%{name}/archive/refs/tags/v%{version}.tar.gz

# This works fine with Fedora and RHEL, but breaks the SUSE build:
# ExclusiveArch: %{rust_arches}

BuildRequires: rust
BuildRequires: cargo

%description
%{summary}

# Disable debugging packages. RPM looks for them even though none are created,
# and that breaks the build if you don't set this option.
%global debug_package %{nil}

%prep
# Unpack the sources.
%setup -q

%build
# Build the binary.
cargo build --release

%install
# Clean up previous artifacts.
rm -rf %{buildroot}
# Prepare the target directory.
mkdir -p %{buildroot}%{_bindir}
# Install the binary into the chroot environment.
install -m 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}
# An alternative way to install the binary using cargo.
# cargo install --path . --root %{buildroot}/usr

%clean
rm -rf %{buildroot}

%files
# Pick documentation and license files from the source directory.
%doc README.md
#%doc CHANGELOG.md
%license LICENSE
# Pick the binary from the virtual, chroot system.
%{_bindir}/%{name}
