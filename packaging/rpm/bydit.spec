Name:           bydit
Version:        1.0.0
Release:        1%{?dist}
Summary:        CLI exporter for Reddit posts and comments with filtering and CSV output

License:        MIT
URL:            https://github.com/its-a-unixsystem/bydit
Source0:        %{url}/archive/refs/tags/v%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  rust-packaging
BuildRequires:  pkgconfig(openssl)
BuildRequires:  openssl-devel

Requires:       openssl

%description
Bydit authenticates against Reddit and lets you filter, export, overwrite,
or delete posts and comments directly from the command line.

%prep
%autosetup -n %{name}-%{version}
%cargo_prep

%build
%cargo_build --release --locked

%install
%cargo_install --release --locked
install -Dm0644 README.md %{buildroot}%{_docdir}/%{name}/README.md

%check
%cargo_test --locked

%files
%license README.md
%doc README.md
%{_bindir}/bydit

%changelog
* Thu Nov 13 2025 Thomas <thomas@februus.net> - 1.0.0-1
- Initial RPM packaging for bydit
