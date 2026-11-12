// Authored/unrun source-guide boundary cases; no browser, provider or runtime.
import test from 'node:test';
import assert from 'node:assert/strict';
import {workflowCatalog,publicLink,sourceLink} from '../docs/integrations/showcase/workflows.mjs';
const sha='a'.repeat(64);
const item=()=>({id:'workflow:synthetic',title:'Synthetic guide',description:'Source only',input_output:'Selected input → planned output',availability:'draft_source',source:{repository:'flow',revision:'b'.repeat(40),path:'src/project.rs'},issue:'https://github.com/egohygiene/flow/issues/10',pull_request:null,steps:[{owner:'operator',action:'Review source'}],prerequisites:['Exact selected files'],remaining:['No execution observed']});
const fixture=()=>({schema_version:'flow.workflow-guide-catalog/v1',claim_scope:'source_guides_only',runtime_qualified:false,observed_executions:0,workflows:[item()]});
test('guide count is source-only and positive execution claims refuse',()=>{
 const value=fixture(),parsed=workflowCatalog(value,sha,'selected.json');assert.equal(parsed.rows.length,1);assert.match(parsed.claimScope,/execution not observed/);
 assert.throws(()=>workflowCatalog({...value,runtime_qualified:true},sha,'x'));assert.throws(()=>workflowCatalog({...value,observed_executions:1},sha,'x'));assert.throws(()=>workflowCatalog({...value,verified:true},sha,'x'));
});
test('duplicate identities, traversing source paths and unsafe links refuse',()=>{
 const value=fixture();assert.throws(()=>workflowCatalog({...value,workflows:[item(),item()]},sha,'x'));
 for(const path of ['../private','/absolute','src/../private','src\\private'])assert.throws(()=>sourceLink({...item().source,path}));
 for(const link of ['javascript:alert(1)','https://github.com.evil/egohygiene/flow/issues/10','https://github.com/egohygiene/flow/issues/10?token=secret'])assert.throws(()=>publicLink(link,'issues'));
});
test('pinned source links keep Unicode as encoded path data',()=>{
 const link=sourceLink({...item().source,path:'docs/作品 guide.md'});assert.match(link,/^https:\/\/github\.com\/egohygiene\/flow\/blob\/[a-f0-9]{40}\//);assert.ok(link.endsWith('docs/%E4%BD%9C%E5%93%81%20guide.md'));
});
