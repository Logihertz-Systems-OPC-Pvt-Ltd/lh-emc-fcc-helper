Name:           lh-emc-fcc-helper
Version:        0.1.0
Release:        1%{?dist}
Summary:        lh-emc-fcc - Maxwell Near-Field H-Probe to Far-Field Radiated Electric Field
License:        Apache-2.0
URL:            https://github.com/Logihertz-Systems-OPC-Pvt-Ltd/lh-emc-fcc-helper
Source0:        %{name}-%{version}-src.tar.gz
Packager:       Sameer Tawade <ops@logihertz.com>
BuildArch:      %{_arch}

%description
Pure-Rust hardware EMC pre-compliance calculator translating near-field H-probe scans into 3m/10m far-field radiated emissions against FCC Part 15B and CISPR 32 Class A/B limits with clock harmonic & TEMPEST analysis.

%prep
%autosetup -n %{name}-0.1.0

%build
# Industrial clean-room build with zero external runtime dependencies

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_docdir}/%{name}
mkdir -p %{buildroot}%{_mandir}/man1

if [ -f bin/lh-emc-fcc ]; then
    install -m 0755 bin/lh-emc-fcc %{buildroot}%{_bindir}/lh-emc-fcc
elif [ -f lh-emc-fcc ]; then
    install -m 0755 lh-emc-fcc %{buildroot}%{_bindir}/lh-emc-fcc
fi

install -m 0644 README.md %{buildroot}%{_docdir}/%{name}/README.md
install -m 0644 SPEC.md %{buildroot}%{_docdir}/%{name}/SPEC.md
install -m 0644 LICENSE %{buildroot}%{_docdir}/%{name}/LICENSE
install -m 0644 NOTICE %{buildroot}%{_docdir}/%{name}/NOTICE

%files
%doc %{_docdir}/%{name}/README.md
%doc %{_docdir}/%{name}/SPEC.md
%doc %{_docdir}/%{name}/NOTICE
%license %{_docdir}/%{name}/LICENSE

%changelog
* Mon Nov 16 2026 Sameer Tawade <ops@logihertz.com> - 0.1.0-1
- Initial production release for Logihertz Systems Ecosystem and STRUCT OS.
