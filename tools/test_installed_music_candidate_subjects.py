#!/usr/bin/env python3
"""Read-only admission observers; synthetic binaries are never executed."""
import copy
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import installed_music_candidate_subjects as music
import test_installed_candidate_subjects as old

cli = music.cli


class Fixture:
    def __init__(self, root):
        self.old = old.Fixture(root)
        f = self.old
        f.row("optiflow")["contracts"] = []; f.tool("optiflow")["contracts"] = []
        f.emit()  # Preserve this original two-provider package before supplementing.
        self.value = {key: value for key, value in f.value.items() if key in music.FIELDS}
        self.value["schema_version"] = "flow.installed-candidate-selection/v2"
        self.value["flow_product_equivalence"] = str(root / "equivalence.json")
        self.value.update(flow_selected_source_revision="5"*40,flow_selected_source_tree="6"*40)
        self.equivalence = {"schema_version":"flow.product-input-equivalence/v1", "build_revision":old.FLOW_REV,
            "build_tree":old.FLOW_TREE,"selected_revision":"5"*40,"selected_tree":"6"*40,
            "excluded_prefixes":["docs/","tools/"],"changed_paths":["tools/observer.py"],"product_changed_paths":[],
            "docs_or_tools_embedded_in_runtime":False}
        self.cohort = copy.deepcopy(f.cohort)
        ani = old.elf("synthetic Aniflow, never executed")
        description = {"schema_version":"aniflow.release-contract/v1","tool":"aniflow","version":"0.3.0",
            "profiles":[{"id":"music-video-source-v1","source_mutation":False}]}
        path = f.root / "bin/aniflow"; path.write_bytes(ani)
        contract = f.root / "contracts/aniflow/00.json"; contract.parent.mkdir();contract.write_bytes(old.encoded(description))
        row = copy.deepcopy(f.row("renderflow")); row.update(id=music.ANI_ROW,tool="aniflow",profile="music-video-source-v1")
        row["artifact"] = {"version":"0.3.0","source_revision":"7"*40,"name":"aniflow",**old.identity(ani),
            "release_url":"https://github.com/egohygiene/aniflow/commit/"+"7"*40}
        row["contracts"] = [{"identifier":"aniflow.release-contract/v1","sha256":cli.sha(contract.read_bytes())}]
        self.cohort["rows"] = [f.row("flow"), row, f.row("renderflow"), f.row("optiflow")]
        self.cohort["retained_rows"] = [r["id"] for r in self.cohort["rows"]]
        tool = {"tool":"aniflow","row_id":music.ANI_ROW,"executable":{"path":cli.native_path(root/'originals/aniflow'),**old.identity(ani)},
            "contracts":[{"identifier":"aniflow.release-contract/v1","file":{"path":cli.native_path(root/'originals/aniflow.json'),**old.identity(contract.read_bytes())}}],"extras":[]}
        self.intent = copy.deepcopy(f.intent);self.intent["tools"] = [f.tool("flow"), tool, f.tool("renderflow"), f.tool("optiflow")]
        self.receipt = copy.deepcopy(f.receipt)
        self.observations = {}
        for name, revision, tree, profile, version, descriptor in [
            ("aniflow","7"*40,"8"*40,"music-video-source-v1","0.3.0",contract),
            ("renderflow",old.RENDER_REV,old.RENDER_TREE,"canonical-build-v1","0.3.0-rc.1",f.root/'contracts/renderflow/00.json')]:
            self.value.update({name+"_source_revision":revision,name+"_source_tree":tree,
                name+"_binary_sha256":music.previous.pin(f.root/'bin'/name)["sha256"],
                name+"_contract_sha256":music.previous.pin(descriptor)["sha256"],name+"_row_id":music.ANI_ROW if name=='aniflow' else music.RENDER_ROW,
                name+"_candidate_observation":str(root/(name+'-capture.json'))})
            self.observations[name] = {"schema_version":"flow.provider-candidate-capture/v1","tool":name,"target":music.TARGET,
                "source":{"revision":revision,"tree":tree},"binary":music.previous.pin(f.root/'bin'/name),
                "contract":{"identifier":name+".release-contract/"+('v1' if name=='aniflow' else 'v2'),**music.previous.pin(descriptor)},
                "version":version,"profile":"release","build_command":["cargo","build","--release"],"rustc":"rustc 1.85.0 (synthetic)",
                "release_published":False,"publisher_authenticated":False,"native_matrix_qualified":False}
        self.value['provider_supplement'] = str(root/'supplement.json')
        self.emit()

    def emit(self):
        f=self.old
        for key, data in [("flow_product_equivalence",self.equivalence),
                *[(name+"_candidate_observation",value) for name,value in self.observations.items()]]:
            raw=old.encoded(data);Path(self.value[key]).write_bytes(raw);self.value[key+"_sha256"]=cli.sha(raw)
        supplement=copy.deepcopy(self.cohort);supplement['rows']=supplement['rows'][1:];supplement['retained_rows']=[r['id'] for r in supplement['rows']]
        raw=old.encoded(supplement);Path(self.value['provider_supplement']).write_bytes(raw);self.value['provider_supplement_sha256']=cli.sha(raw)
        raw=old.encoded(self.cohort);(f.root/'cohort.json').write_bytes(raw)
        self.intent['cohort']={'path':cli.native_path(Path(self.value['provider_supplement'])),**old.identity(raw)}
        raw=old.encoded(self.intent);(f.root/'installation-intent.json').write_bytes(raw);self.receipt['intent_sha256']=cli.sha(raw)
        raw=old.encoded(self.receipt);(f.root/'installation.json').write_bytes(raw);self.value['installation_receipt_sha256']=cli.sha(raw)

    def admit(self): return music.InstalledMusicCandidate(self.value)


class Cases(unittest.TestCase):
    def setUp(self):
        for key,data in [('PROVIDER_SHA',old.OPTI),('BUNDLE_SHA',old.BUNDLE)]:
            self.enterContext(patch.object(cli,key,cli.sha(data)))
        self.temp=self.enterContext(tempfile.TemporaryDirectory());self.f=Fixture(Path(self.temp))

    def test_four_tool_supplement_admits_without_rewriting_original_flow_package(self):
        original=music.previous.pin(self.f.old.package/music.PACKAGE.MANIFEST)
        subject=self.f.admit()
        self.assertEqual(subject.aniflow,self.f.old.root/'bin/aniflow')
        self.assertEqual(music.previous.pin(self.f.old.package/music.PACKAGE.MANIFEST),original)
        self.assertEqual(subject.flow_identity['actual_build_revision'],old.FLOW_REV)
        self.assertIn('separately',subject.flow_identity['scope'])

    def test_v1_stays_closed_and_v2_is_explicit(self):
        with self.assertRaises(ValueError): music.previous.selection(self.f.value)
        with self.assertRaises(ValueError): music.selection(self.f.old.value)

    def test_duplicate_or_missing_supplement_provider_refuses_before_projection(self):
        for mutate in [lambda r:r.pop(),lambda r:r.append(copy.deepcopy(r[0]))]:
            f=copy.deepcopy(self.f.cohort);s=copy.deepcopy(f);s['rows']=s['rows'][1:];mutate(s['rows'])
            with self.assertRaises(ValueError):music.provider_projection(s,f)

    def test_changed_capture_source_or_inferred_qualification_refuses(self):
        for change in [lambda o:o['source'].update(tree='9'*40),lambda o:o.update(native_matrix_qualified=True)]:
            original=copy.deepcopy(self.f.observations['aniflow']);change(self.f.observations['aniflow']);self.f.emit()
            with self.assertRaises(ValueError):self.f.admit()
            self.f.observations['aniflow']=original

    def test_unknown_selection_field_cannot_be_an_authority_cache(self):
        self.f.value['verified']='true'
        with self.assertRaises(ValueError):self.f.admit()

    def test_changed_installed_binary_and_contract_refuse(self):
        for path in [self.f.old.root/'bin/aniflow',self.f.old.root/'contracts/aniflow/00.json']:
            raw=path.read_bytes();path.write_bytes(raw+b'changed')
            with self.assertRaises(ValueError):self.f.admit()
            path.write_bytes(raw)

    def test_product_equivalence_cannot_hide_a_rust_change(self):
        self.f.equivalence['product_changed_paths']=['src/native/aniflow.rs'];self.f.emit()
        with self.assertRaises(ValueError):self.f.admit()

    def test_product_equivalence_cannot_substitute_selected_source(self):
        self.f.equivalence['selected_revision']='8'*40;self.f.emit()
        with self.assertRaises(ValueError):self.f.admit()

    def test_frozen_selection_mutation_is_not_readmitted(self):
        path=Path(self.temp)/'selection.json';path.write_bytes(old.encoded(self.f.value))
        subject=music.InstalledMusicCandidate.load(path);path.write_bytes(path.read_bytes()+b' ')
        with self.assertRaises(ValueError):subject.reobserve()


if __name__=='__main__': unittest.main()
