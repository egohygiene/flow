#!/usr/bin/env python3
"""Bounded public local-input entry proof using existing installed CLI owners.

No router/installer/media engine, acquisition, release or human review is added.
The original Flow build and installation selections remain their exact subjects.
"""
import argparse
import copy
from datetime import datetime, timezone
from pathlib import Path
import re

import check_native_publication as publication
import check_installed_music_video as music
import check_released_read_only as readonly
from installed_music_candidate_subjects import InstalledMusicCandidate

pin, put, refusal = publication.pin, publication.put, publication.refusal
canonical, document, require, sha, command, native_path = (readonly.canonical,
    readonly.document, readonly.require, readonly.sha, readonly.command, readonly.native_path)
LOCAL_LIMITS = {"maximum_entries": 64, "maximum_files": 16, "maximum_depth": 4,
    "maximum_file_bytes": 2000000, "maximum_total_bytes": 8000000, "deadline_ms": 10000}
NATIVE_LIMITS = {"timeout_ms": 120000, "max_input_bytes": 2000000, "max_output_bytes": 16000000,
    "max_total_output_bytes": 32000000, "max_artifacts": 8}
SELECTED_REVISION = "80127ca4f9ad14ebb8e8c5a8f7321f242aa36607"
SELECTED_TREE = "e50ff08a3828b7c5febc6ded5fb3d131aed7a9f2"


def observe_inventory(value, paths):
    require(value["schema_version"] == "flow.local-input-inventory/v1"
        and value["domain_acceptance"] is False and value["execution_input_freeze"] is False
        and "extension_is_not_media_proof" in value["content_type"]
        and len(value["files"]) == len(paths)
        and value["total_bytes"] == sum(pin(p)["byte_count"] for p in paths),
        "local inspection lost inputs or promoted filesystem bytes to domain acceptance")
    for ordinal, (item, path) in enumerate(zip(value["files"], paths)):
        require(type(item["ordinal"]) is int and item["ordinal"] == ordinal
            and item["path"] == native_path(path) and item["sha256"] == pin(path)["sha256"]
            and type(item["byte_count"]) is int and item["byte_count"] == pin(path)["byte_count"]
            and type(item["filesystem_id"]) is int and type(item["file_id"]) is int,
            "local inspection path/order/current identity differs")
    return value


def observe_native_plan(value, saved, paths, selection):
    require(value["schema_version"] == "flow.local-input-native-plan/v1"
        and value["release_qualified"] is False, "local wrapper promoted candidate qualification")
    observe_inventory(value["inventory"], paths)
    plan = value["command_plan"]
    require(document(readonly.OWNER_IO.regular(saved, 16777216)) == plan
        and plan["schema_version"] == "flow.native-plan/v1"
        and plan["request"] == selection and plan["sources"] == [pin(p) for p in paths]
        and plan["release_evidence"] == "operator_pinned_source_candidate; publication_and_qualification_not_established",
        "wrapper saved an unrelated/outer executable plan or changed selected scope")
    return plan


def build_equivalence(path, digest, subject):
    raw = readonly.OWNER_IO.regular(path, 131072)
    require(sha(raw) == digest, "explicit current-source equivalence pin differs")
    value = document(raw)
    require(value["schema_version"] == "flow.local-input-build-equivalence/v1"
        and value["actual_build_revision"] == subject.selection["flow_source_revision"]
        and value["actual_build_tree"] == subject.selection["flow_source_tree"]
        and value["selected_revision"] == SELECTED_REVISION and value["selected_tree"] == SELECTED_TREE
        and value["runtime_inputs_equal"] is True and value["changed_compilation_inputs"] == []
        and value["embedded_nonruntime_files"] is False
        and value["permitted_nonruntime_changes"] == ["docs/", "tools/", "Taskfile.yml"]
        and all(type(p) is str and (p.startswith(("docs/", "tools/")) or p == "Taskfile.yml")
            and ".." not in Path(p).parts for p in value["changed_paths"]),
        "current source cannot reuse this older measured Flow executable")
    return {"schema_version": value["schema_version"], "actual_build_revision": value["actual_build_revision"],
        "actual_build_tree": value["actual_build_tree"], "selected_revision": value["selected_revision"],
        "selected_tree": value["selected_tree"], "runtime_inputs_equal": True, "observation_sha256": digest,
        "scope": "unchanged existing executable; separate local Git input comparison, no new build or authentication"}


def local_request(input, profile, limits=None):
    return {"schema_version": "flow.local-input-request/v1", "input": input, "profile": profile,
        "provider_executable": None, "run_directory": None, "limits": limits or LOCAL_LIMITS}


def run(selection_path, equivalence_path, equivalence_sha256, video_input, recipe_input, video_tools, output):
    subject = InstalledMusicCandidate.load(selection_path)
    equivalence = build_equivalence(equivalence_path, equivalence_sha256, subject)
    require(output.is_absolute() and output.parent.resolve() == output.parent and not output.exists(),
        "fresh canonical local-input proof root required")
    output.mkdir(mode=0o700); logs=output/"commands";logs.mkdir(mode=0o700)
    cwd=output/"empty cwd 作品";cwd.mkdir(mode=0o700)
    source=output/"mixed source 作品";source.mkdir(mode=0o700)
    originals={"video":pin(video_input),"recipe":pin(recipe_input)}
    video=source/"ordinary source 作品.mp4";video.write_bytes(readonly.OWNER_IO.regular(video_input,2000000))
    recipe=output/"source recipe.json";recipe.write_bytes(readonly.OWNER_IO.regular(recipe_input,131072))
    pages=[source/"page 001.png",source/"page 002.png"]
    for path,color in zip(pages,[(30,70,130),(180,90,40)]):path.write_bytes(publication.png(color))
    note=source/"not media.txt";note.write_bytes(b"synthetic ordinary bytes, not media proof\n")
    source_pins={p.name:pin(p) for p in [video,*pages,note]}
    cases=[];routes=[]
    before_suite=subject.observe_status(output,"suite-before")

    def invoke(label,args,code=0,diagnostic=None,outcome=None,expected_command=None):
        actual,stdout,stderr=command([str(subject.flow),*map(str,args),"--format","json"],cwd=cwd,timeout=145,max_output=16777216)
        (logs/(label+".stdout.json")).write_bytes(stdout);(logs/(label+".stderr")).write_bytes(stderr)
        expected_outcome=outcome or ({0:"complete",4:"unavailable",7:"recovery_required",9:"partial"}.get(code,"refused"))
        value=readonly.envelope(stdout,stderr,actual,expected_command or args[0],code,expected_outcome)
        if diagnostic:refusal(value,diagnostic)
        else:require(value["diagnostics"]==[] and type(value["result"]) is dict,"unexpected local-input diagnostic")
        cases.append({"case":label,"command":value["command"],"exit_code":actual,"outcome":value["outcome"],
            "stdout_sha256":sha(stdout),"stderr_sha256":sha(stderr)})
        return value["result"]

    # Mixed ordinary files are inspected as octets, never silently routed by extension.
    ordered=sorted([video,*pages,note],key=lambda p:p.name)
    observe_inventory(invoke("mixed-directory-inspection",["inspect","--input",source]),ordered)
    observe_inventory(invoke("single-file-inspection",["inspect","--input",video]),[video])

    def native_selection(name,tool,inputs):
        root=output/(name+" run");configuration=recipe
        if tool=="renderflow":
            configuration=output/(name+" configuration.json")
            put(configuration,publication.configuration(pages,root,"epub",Path("/unused/no-pdf-provider")))
        provider=subject.aniflow if tool=="aniflow" else subject.provider
        descriptor=subject.aniflow_descriptor if tool=="aniflow" else subject.descriptor
        selected={"schema_version":"flow.native-request/v1","cohort":native_path(subject.cohort_path),
            "cohort_sha256":pin(subject.cohort_path)["sha256"],"row_id":subject.selection[tool+"_row_id"],
            "provider_executable":native_path(provider),"release_descriptor":native_path(descriptor),
            "inputs":[native_path(p) for p in inputs],"configuration":native_path(configuration),
            "run_directory":native_path(root),"tool_directories":[native_path(video_tools)] if tool=="aniflow" else [],
            "limits":NATIVE_LIMITS}
        return root,selected

    def wrapper(name,request,selected):
        path=output/(name+" wrapper.json")
        put(path,{"schema_version":"flow.local-input-native-request/v1","request":request,"selection":selected})
        return path

    def positive(name,tool,inputs,flags):
        root,selected=native_selection(name,tool,inputs);selection_file=output/(name+" native-selection.json");put(selection_file,selected)
        local=local_request({"kind":"file","path":native_path(inputs[0])} if len(inputs)==1 else
            {"kind":"ordered_collection","members":[native_path(p) for p in inputs]},
            "aniflow_temporal" if tool=="aniflow" else "renderflow_publication")
        wrapped=wrapper(name,local,selected);plan=output/(name+" plan.json");approval=output/(name+" approval.json")
        if flags:
            args=["plan","--input",inputs[0],"--profile","aniflow-temporal","--native-selection",selection_file,"--output",plan]
        else:args=["plan","--request",wrapped,"--output",plan]
        planned=invoke(name+"-wrapper-plan",args);observe_native_plan(planned,plan,inputs,selected)
        require(not root.exists(),"local planning launched native provider or created a run")
        invoke(name+"-candidate-ack-refusal",["approve","--plan",plan,"--approved-by","user-authorized synthetic execution",
            "--acknowledge-trusted-unconfined","--output",approval],5,
            ("approval_required","exact plan approval must acknowledge trusted-unconfined, unqualified native candidate execution",None))
        require(not approval.exists() and not root.exists(),"refused acknowledgement wrote runtime state")
        invoke(name+"-approve",["approve","--plan",plan,"--approved-by","user-authorized synthetic execution",
            "--acknowledge-trusted-unconfined","--acknowledge-unqualified-candidate","--output",approval])
        value=invoke(name+"-run",["run","--plan",plan,"--approval",approval])
        receipt=music.accepted(value,root,video,subject.aniflow) if tool=="aniflow" else publication.accepted(value,root,"epub",pages)[0]
        before=music.inventory(root)
        status=invoke(name+"-status",["status","--run-directory",root]);require(status["receipt"]==receipt,"wrapper status receipt changed")
        resumed=invoke(name+"-completed-resume",["resume","--run-directory",root,"--native-run-directory",receipt["native_run"]["value"]])
        require(resumed==receipt and music.inventory(root)==before,"completed wrapper route replayed/changed native state")
        routes.append({"tool":tool,"entry":"flags_native_selection" if flags else "closed_local_native_wrapper",
            "plan_sha256":planned["command_plan"]["plan_sha256"],"accepted_receipt_sha256":pin(root/"accepted.json")["sha256"],
            "artifact_count":len(receipt["artifacts"]),"completed_state_preserved":True,
            "release_qualified":False,"creative_approval_inferred":False})

    positive("aniflow single", "aniflow", [video], True)
    positive("renderflow ordered", "renderflow", pages, False)

    def negative(label,input,profile="renderflow_publication",limits=None,change_selection=None,message=None,code=3,diagnostic="invalid_input"):
        root,selected=native_selection(label,"renderflow",pages)
        if change_selection:change_selection(selected)
        path=wrapper(label,local_request(input,profile,limits),selected);plan=output/(label+" forbidden-plan.json")
        invoke(label,["plan","--request",path,"--output",plan],code,(diagnostic,message,None))
        require(not root.exists() and not plan.exists(),"refused local/native selection wrote an executable plan or native root")

    collection=lambda paths:{"kind":"ordered_collection","members":[native_path(p) for p in paths]}
    scope_reason="local input and native selection scopes/overrides differ"
    negative("reordered-local-members",collection(list(reversed(pages))),message=scope_reason)
    negative("mixed-extra-member",collection([*pages,note]),message=scope_reason)
    negative("directory-does-not-expand",{"kind":"directory","path":native_path(source),"recursive":True},
        message="native local route requires a selected file or explicit ordered collection; no directory expansion")
    negative("duplicate-local-member",collection([pages[0],pages[0]]),message="collection paths must be distinct; order is explicit")
    negative("wrong-profile-family",collection(pages),profile="aniflow_temporal",code=3,diagnostic="stale_plan",
        message="native profile or selected input bytes/order differ from local inspection")
    negative("wrong-native-order",collection(list(reversed(pages))),change_selection=lambda s:s["inputs"].reverse(),
        message="native ordered source path differs from selected input")
    small={**LOCAL_LIMITS,"maximum_file_bytes":10,"maximum_total_bytes":100}
    negative("local-file-budget",collection(pages),limits=small,message="input is redirected/aliased/unsupported or exceeds selected bounds")
    negative("missing-native-provider",collection(pages),change_selection=lambda s:s.update(provider_executable=native_path(output/"not-installed")),
        code=3,diagnostic="stale_plan",message="pinned file or release cohort could not be reobserved")
    # A profile name cannot turn an ordinary file into recursive Optiflow scope.
    forbidden=output/"unsupported-opti-plan.json"
    unavailable=invoke("optiflow-file-does-not-widen",["plan","--input",note,"--profile","optiflow-read-only",
        "--provider-executable",subject.optiflow,"--run-directory",output/"unavailable opti run","--output",forbidden],4)
    require(unavailable["availability"]=="unavailable" and unavailable["command_plan"] is None
        and unavailable["reason"]=="pinned Optiflow adapter requires whole recursive directory scope; file/ordered/nonrecursive requests cannot be broadened"
        and not forbidden.exists(),"unsupported Optiflow file became an executable widened plan")
    invoke("inspection-output-inside-source",["plan","--input",source,"--output",source/"forbidden inventory.json"],3,
        ("invalid_input","inspection snapshot output is inside selected directory scope",None))
    require(not (source/"forbidden inventory.json").exists(),"inspection wrote into source scope")

    opti_source=output/"read-only directory 作品";opti_source.mkdir(mode=0o700)
    for name,data in readonly.FIXTURE.items():(opti_source/name).write_bytes(data)
    opti_run=output/"optiflow run";opti_plan=output/"opti scope plan.json";opti_approval=output/"opti approval.json"
    planned=invoke("released-opti-wrapper-plan",["plan","--input",opti_source,"--profile","optiflow-read-only",
        "--provider-executable",subject.optiflow,"--run-directory",opti_run,"--output",opti_plan],9)
    require(planned["availability"]=="scope_plan_only" and planned["inspection_limits_apply_to_native_execution"] is False
        and planned["command_plan"]==document(readonly.OWNER_IO.regular(opti_plan,32768)) and not opti_run.exists(),
        "released Optiflow scope preview became frozen execution or provider invocation")
    invoke("released-opti-approve",["approve","--plan",opti_plan,"--approved-by","user-authorized synthetic read-only execution",
        "--acknowledge-trusted-unconfined","--output",opti_approval])
    completed=invoke("released-opti-run",["run","--plan",opti_plan,"--approval",opti_approval])
    receipt=readonly.completed_status(completed);readonly.retained_receipt(completed,opti_run,receipt);readonly.native_artifacts(receipt,opti_run)
    before=readonly.inventory(opti_run)
    require(readonly.completed_status(invoke("released-opti-status",["status","--run-directory",opti_run]))==receipt,"Opti retained status changed")
    require(readonly.completed_status(invoke("released-opti-completed-resume",["resume","--run-directory",opti_run]))==receipt
        and readonly.inventory(opti_run)==before,"Opti completed resume replayed native work")
    routes.append({"tool":"optiflow","entry":"flags_recursive_directory","release":"0.1.1",
        "planning_availability":"scope_plan_only","inspection_limits_apply_to_native_execution":False,
        "completed_state_preserved":True,"source_mutation_authority":False})
    after_suite=subject.observe_status(output,"suite-after")
    require(not list(cwd.iterdir()) and {"video":pin(video_input),"recipe":pin(recipe_input)}==originals
        and {p.name:pin(p) for p in [video,*pages,note]}==source_pins,"original/generated bytes or empty command cwd changed")
    require(pin(equivalence_path)["sha256"]==equivalence_sha256,"equivalence observation changed during proof")
    report={"schema_version":"flow.installed-local-input-proof/v1","status":"passed","observed_utc":datetime.now(timezone.utc).isoformat(),
        "driver_sha256":pin(Path(__file__).resolve())["sha256"],"flow_build":subject.flow_identity,"current_source_equivalence":equivalence,
        "selection_sha256":subject.selection_sha256,"aniflow_candidate":subject.aniflow_identity,"renderflow_candidate":subject.render_identity,
        "commands":cases,"routes":routes,"suite_before":before_suite,"suite_after":after_suite,"supplied_originals_preserved":True,
        "generated_fixture_bytes_preserved":True,"command_cwd_outside_checkout":True,"empty_cwd_preserved":True,
        "human_reviews":"awaiting","release_published":False,"publisher_authenticated":False,"native_matrix_qualified":False,
        "hosted_ci_dispatched":False,"source_mutation_authority":False,"publication_authority":False}
    put(output/"report.json",report);return report


if __name__=="__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ["selection","equivalence","video","recipe","video-tools","output"]:parser.add_argument("--"+name,type=Path,required=True)
    parser.add_argument("--equivalence-sha256",required=True)
    args=parser.parse_args()
    print(canonical(run(args.selection,args.equivalence,args.equivalence_sha256,args.video,args.recipe,args.video_tools,args.output)).decode())
