import plistlib, sys, zipfile, struct

def verify(path):
    with zipfile.ZipFile(path) as z:
        names = z.namelist()
        plists = [n for n in names if n.startswith('Payload/') and n.count('/') == 2 and n.endswith('.app/Info.plist')]
        assert len(plists) == 1, 'Expected one application Info.plist'
        base = plists[0].removesuffix('Info.plist')
        info = plistlib.loads(z.read(plists[0]))
        assert info['CFBundlePackageType'] == 'APPL'
        assert info['MinimumOSVersion'] == '15.0'
        executable = info['CFBundleExecutable']
        assert '/' not in executable and executable, 'Invalid executable name'
        binary = z.read(base + executable)
        assert binary[:4] == b'\xcf\xfa\xed\xfe', 'Expected 64-bit Mach-O'
        assert struct.unpack_from('<I', binary, 4)[0] == 0x0100000c, 'Expected ARM64'
        assert any(n.endswith('/manifests-v1.json') for n in names), 'Missing catalog'
        assert not any(n.endswith('.dylib') for n in names), 'Unexpected dynamic libraries; review packaging'
    print('IPA structure verified: ARM64 executable, metadata and manifest')
if __name__ == '__main__': verify(sys.argv[1])
