#!/usr/bin/env python3
"""Targeted staging-budget refusal/preparation and retained-record readback.

No native provider is launched. Existing synthetic inputs stay unchanged.
"""
import argparse
import copy
import json
from pathlib import Path

import check_native_publication as native


def run_check(flow, build, prior, output):
    build = native.public_candidate_build(build)
    binary = native.build_binary(flow)
    native.require({"sha256":native.sha(binary),"bytes":len(binary)} == build["binary"], "selected build differs")
    native.require(output.is_absolute() and output.parent.resolve(strict=True) == output.parent, "canonical fresh output parent required")
    output.mkdir(mode=0o700)
    selected = output/'flow'; selected.write_bytes(binary); selected.chmod(0o700)
    logs = output/'commands'; logs.mkdir(mode=0o700)
    prior_report = native.document(native.OWNER_IO.regular(prior/'report.json',1048576))
    native.require(prior_report['schema_version']=='flow.static-publication-cli-pilot/v1' and prior_report['status']=='passed', "existing scoped static pilot required")
    project = native.document(native.OWNER_IO.regular(prior/'epub-project.json',1048576))
    sources = [Path(s['file']['path']['value']) for s in project['branches'][0]['sources']]
    originals = [native.pin(p) for p in sources]
    native.require(len(originals)==2 and sum(p['byte_count'] for p in originals)==438
        and all(p.is_relative_to(prior/'source 作品 with spaces') for p in sources), "selected small synthetic fixture required")
    observations=[]
    def invoke(label, args, code=0, outcome='complete', diagnostic=None):
        actual, stdout, stderr=native.command([str(selected),*args,'--format','json'],cwd=output,timeout=30,max_output=16777216)
        (logs/(label+'.stdout.json')).write_bytes(stdout);(logs/(label+'.stderr')).write_bytes(stderr)
        value=native.envelope(stdout,stderr,actual,args[0],code,outcome)
        if diagnostic: native.refusal(value,diagnostic)
        else: native.require(value['diagnostics']==[], 'unexpected diagnostic')
        observations.append({'case':label,'command':args[0],'exit_code':actual,'outcome':outcome,'stdout_sha256':native.sha(stdout)})
        return value['result']
    # Existing tiny default-budget preparations retain their original identity.
    old_workspace=Path(project['workspace']['value'])
    old_record=native.pin(old_workspace/'prepared-static-project.json')
    old_status=invoke('old-default-status',['publication-project-status','--workspace',str(old_workspace),
        '--record-sha256',old_record['sha256']],9,'partial')
    native.require(old_status['all_required_artifacts_observed'] is True and old_status['creative_review_pending'] is True,
        'default-budget compatibility changed retained acceptance')
    refusal=('invalid_input','static collection copies exceed maximum_staged_bytes',None)
    over=copy.deepcopy(project);over['maximum_staged_bytes']=437;over['workspace']=native.native_path(output/'over-budget-workspace')
    over_path=output/'over-budget-project.json';native.put(over_path,over)
    for verb in ['preview','prepare']:
        invoke('over-budget-'+verb,['publication-project-'+verb,'--project',str(over_path)],3,'refused',refusal)
        native.require(not (output/'over-budget-workspace').exists(),'over-budget operation wrote a workspace')
    repeated=copy.deepcopy(project); repeated['maximum_staged_bytes']=875
    repeated['workspace']=native.native_path(output/'repeated-workspace')
    duplicate=copy.deepcopy(repeated['branches'][0]);duplicate['id']='fr';duplicate['locale']='fr';repeated['branches'].append(duplicate)
    repeated_path=output/'repeated-project.json';native.put(repeated_path,repeated)
    invoke('repeated-references-prepare',['publication-project-prepare','--project',str(repeated_path)],3,'refused',refusal)
    native.require(not (output/'repeated-workspace').exists(),'duplicate-reference aggregate bypassed staging budget')
    positive=copy.deepcopy(project);positive['maximum_staged_bytes']=438
    positive['workspace']=native.native_path(output/'exact-budget-workspace')
    positive_path=output/'exact-budget-project.json';native.put(positive_path,positive)
    preview=invoke('exact-budget-preview',['publication-project-preview','--project',str(positive_path)])
    native.require(not (output/'exact-budget-workspace').exists() and preview['provider_invoked'] is False,'preview wrote or executed')
    prepared=invoke('exact-budget-prepare',['publication-project-prepare','--project',str(positive_path)])
    native.require(prepared['provider_invoked'] is False and prepared['approvals_generated'] is False,'preparation launched or approved')
    copied=[Path(p['value']) for p in prepared['branches'][0]['plan']['request']['inputs']]
    native.require(sum(native.pin(p)['byte_count'] for p in copied)==438
        and [(native.pin(p)['sha256'],native.pin(p)['byte_count']) for p in copied]==[(p['sha256'],p['byte_count']) for p in originals],
        'staging copy bytes/order exceed or differ from exact budget')
    status=invoke('exact-budget-prepared-status',['publication-project-status','--workspace',str(output/'exact-budget-workspace'),
        '--record-sha256',prepared['record']['sha256']],9,'partial')
    native.require(status['all_required_artifacts_observed'] is False and status['branches'][0]['state']=='not_started'
        and status['creative_review_pending'] is True,'preparation became native/review acceptance')
    native.require([native.pin(p) for p in sources]==originals,'original source bytes changed')
    return {'schema_version':'flow.static-staging-budget-check/v1','status':'passed','flow_build':build,
        'driver_sha256':native.pin(Path(__file__).resolve())['sha256'],'prior_static_report_sha256':native.pin(prior/'report.json')['sha256'],
        'commands':observations,'default_budget_bytes':268435456,'maximum_budget_bytes':68719476736,
        'single_collection_staged_bytes':438,'repeated_locale_staged_bytes':876,
        'over_budget_workspace_created':False,'repeated_workspace_created':False,'exact_budget_staged_bytes':438,
        'originals_unchanged':True,'retained_default_record_compatible':True,'provider_invoked':False,
        'native_qualified':False,'creative_approval_inferred':False,'publication_authority':False,
        'scope':'targeted aggregate staging guard and private preparation; original22-call native proof preserved'}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['flow','flow-build-evidence','prior-static-pilot','output']:parser.add_argument('--'+name,type=Path,required=True)
    args=parser.parse_args()
    result=run_check(args.flow,native.document(native.OWNER_IO.regular(args.flow_build_evidence,65536)),args.prior_static_pilot,args.output)
    native.put(args.output/'report.json',result)
    print(json.dumps({'status':result['status'],'commands':len(result['commands']),'provider_invoked':False}))


if __name__=='__main__':main()
