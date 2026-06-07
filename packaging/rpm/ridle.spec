Name:           ridle
Version:        TEMPLATE_VERSION
Release:        1%{?dist}
Summary:        A project template for creating unified local terminal utilities in Rust
License:        MIT
URL:            https://github.com/local76/rIdle
Source0:        %{name}-%{version}.tar.gz

%description
A project template for creating unified local terminal utilities in Rust.

%prep
%setup -q

%build
cargo build --release --locked

%install
rm -rf $RPM_BUILD_ROOT
install -d $RPM_BUILD_ROOT/%{_bindir}
install -d $RPM_BUILD_ROOT/%{_datadir}/applications
install -d $RPM_BUILD_ROOT/%{_datadir}/pixmaps
install -m 755 target/release/ridle $RPM_BUILD_ROOT/%{_bindir}/ridle
install -m 644 packaging/desktop/ridle.desktop $RPM_BUILD_ROOT/%{_datadir}/applications/ridle.desktop
install -m 644 assets/brand/app_icon.png $RPM_BUILD_ROOT/%{_datadir}/pixmaps/ridle.png

%files
%{_bindir}/ridle
%{_datadir}/applications/ridle.desktop
%{_datadir}/pixmaps/ridle.png
