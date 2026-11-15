#!/usr/bin/env python3
"""Independent local-wrapper observation controls; no provider is executed."""
import copy
from pathlib import Path
import tempfile
import unittest

import check_installed_local_inputs as proof


class Cases(unittest.TestCase):
    def setUp(self):
        self.temp=self.enterContext(tempfile.TemporaryDirectory());self.root=Path(self.temp)
        self.paths=[self.root/'second 作品.png',self.root/'first.txt']
        for path,data in zip(self.paths,[b'second fixture',b'first fixture']):path.write_bytes(data)
        self.inventory={'schema_version':'flow.local-input-inventory/v1','domain_acceptance':False,'execution_input_freeze':False,
            'content_type':'unclassified_octets; extension_is_not_media_proof','total_bytes':sum(p.stat().st_size for p in self.paths),
            'files':[{'ordinal':i,**proof.pin(p),'filesystem_id':1,'file_id':i+1} for i,p in enumerate(self.paths)]}

    def test_inspection_cannot_drop_mixed_member_reorder_or_claim_media_acceptance(self):
        proof.observe_inventory(self.inventory,self.paths)
        changes=[lambda v:v['files'].reverse(),lambda v:v['files'].pop(),lambda v:v.update(domain_acceptance=True),
            lambda v:v.update(execution_input_freeze=True),lambda v:v['files'][0].update(ordinal=False)]
        for change in changes:
            altered=copy.deepcopy(self.inventory);change(altered)
            with self.assertRaises(ValueError):proof.observe_inventory(altered,self.paths)

    def test_only_the_exact_inner_command_plan_is_saved_for_execution(self):
        selected={'schema_version':'flow.native-request/v1','fixture':'never executed'}
        plan={'schema_version':'flow.native-plan/v1','request':selected,'sources':[proof.pin(p) for p in self.paths],
            'release_evidence':'operator_pinned_source_candidate; publication_and_qualification_not_established'}
        result={'schema_version':'flow.local-input-native-plan/v1','release_qualified':False,'inventory':self.inventory,'command_plan':plan}
        saved=self.root/'plan.json';proof.put(saved,plan);proof.observe_native_plan(result,saved,self.paths,selected)
        saved.write_bytes(proof.canonical(result))
        with self.assertRaises(ValueError):proof.observe_native_plan(result,saved,self.paths,selected)
        saved.write_bytes(proof.canonical(plan));changed=copy.deepcopy(result);changed['command_plan']['sources'].reverse()
        with self.assertRaises(ValueError):proof.observe_native_plan(changed,saved,self.paths,selected)

    def test_equivalence_does_not_relabel_binary_or_accept_a_changed_runtime(self):
        class Subject:selection={'flow_source_revision':'a'*40,'flow_source_tree':'b'*40}
        value={'schema_version':'flow.local-input-build-equivalence/v1','actual_build_revision':'a'*40,'actual_build_tree':'b'*40,
            'selected_revision':proof.SELECTED_REVISION,'selected_tree':proof.SELECTED_TREE,'runtime_inputs_equal':True,
            'changed_compilation_inputs':[],'embedded_nonruntime_files':False,'permitted_nonruntime_changes':['docs/','tools/','Taskfile.yml'],
            'changed_paths':['Taskfile.yml','docs/example.md','tools/proof.py'],'private_unknown':'not projected'}
        path=self.root/'equivalence.json';proof.put(path,value)
        observed=proof.build_equivalence(path,proof.pin(path)['sha256'],Subject())
        self.assertEqual(observed['actual_build_revision'],'a'*40);self.assertNotIn('private_unknown',observed)
        for change in [lambda v:v.update(runtime_inputs_equal=False),lambda v:v['changed_paths'].append('src/local_input.rs')]:
            altered=copy.deepcopy(value);change(altered);path.write_bytes(proof.canonical(altered))
            with self.assertRaises(ValueError):proof.build_equivalence(path,proof.pin(path)['sha256'],Subject())


if __name__=='__main__':unittest.main()
