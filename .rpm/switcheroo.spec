%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: switcheroo
Summary: A CLI and GUI for the Nintendo Switch RCM exploit
Version: @@VERSION@@
Release: @@RELEASE@@%{?dist}
License: GPLv2+
Group: Applications/System
Source0: %{name}-%{version}.tar.gz

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

%description
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}

# install files
mkdir -p %buildroot/usr/share/bash-completion/completions 
cp ../../../../../extra/completions/switcheroo.bash %buildroot/usr/share/bash-completion/completions/switcheroo
mkdir -p %buildroot/usr/share/fish/vendor_completions.d 
cp ../../../../../extra/completions/switcheroo.fish %buildroot/usr/share/fish/vendor_completions.d/switcheroo.fish
mkdir -p %buildroot/usr/share/zsh/vendor-completions/
cp ../../../../../extra/completions/_switcheroo %buildroot/usr/share/zsh/vendor-completions/_switcheroo

mkdir -p %buildroot/usr/share/applications
cp ../../../../../extra/linux/io.ebudd.Switcheroo.desktop %buildroot/usr/share/applications/io.ebudd.Switcheroo.desktop
mkdir -p %buildroot/usr/share/icons/hicolor/512x512/apps
cp ../../../../../extra/logo/io.ebudd.Switcheroo.png %buildroot/usr/share/icons/hicolor/512x512/apps/io.ebudd.Swithcheroo.png
mkdir -p %buildroot/usr/share/metainfo
cp ../../../../../extra/linux/io.ebudd.Switcheroo.appdata.xml %buildroot//usr/share/metainfo/io.ebudd.Switcheroo.appdata.xml

%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/*
/usr/share/*
