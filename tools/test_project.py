import importlib.util, json, pathlib, plistlib, struct, tempfile, unittest, zipfile
R = pathlib.Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('ipa', R/'tools/verify_ipa.py')
ipa = importlib.util.module_from_spec(spec); spec.loader.exec_module(ipa)
class ProjectTests(unittest.TestCase):
    def test_config(self):
        p = plistlib.loads((R/'ios/IW4Pocket/Info.plist').read_bytes())
        self.assertEqual(p['UIRequiredDeviceCapabilities'], ['arm64'])
        self.assertTrue(all('Landscape' in s for s in p['UISupportedInterfaceOrientations']))
        y = (R/'ios/project.yml').read_text()
        self.assertIn('IPHONEOS_DEPLOYMENT_TARGET: "15.0"',y)
        self.assertIn('-llauncher',y)
        self.assertEqual(json.loads((R/'ios/IW4Pocket/Resources/manifests-v1.json').read_text()),[])
    def test_ipa_structure(self):
        with tempfile.TemporaryDirectory() as d:
            p = pathlib.Path(d)/'fixture.ipa'
            def make(binary):
                with zipfile.ZipFile(p,'w') as z:
                    z.writestr('Payload/Test.app/Info.plist',plistlib.dumps(dict(CFBundlePackageType='APPL',MinimumOSVersion='15.0',CFBundleExecutable='Test')))
                    z.writestr('Payload/Test.app/Test',binary)
                    z.writestr('Payload/Test.app/manifests-v1.json','[]')
            make(b'\xcf\xfa\xed\xfe'+struct.pack('<I',0x0100000c)+bytes(24)); ipa.verify(p)
            make(b'not an executable')
            with self.assertRaises(AssertionError): ipa.verify(p)
if __name__ == '__main__': unittest.main()
