// Source-guide parsing only; never a capability or execution-evidence validator.
import {text} from './model.mjs';
const repos=new Set(['flow','aniflow','renderflow','optiflow']);
function closed(value,keys) {
  if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==keys.length||!keys.every(k=>Object.hasOwn(value,k)))throw new Error('Closed workflow guide record required.');
}
function strings(value,max=32) {if(!Array.isArray(value)||!value.length||value.length>max)throw new Error('Bounded nonempty guide text list required.');return value.map(v=>text(v,2048));}
export function publicLink(value,type) {
  if(typeof value!=='string'||value.length>512||/[\u0000-\u0020\u007f]/.test(value)||!new RegExp(`^https://github\\.com/egohygiene/(flow|aniflow|renderflow|optiflow)/${type}/[1-9][0-9]*$`).test(value))throw new Error('Only explicit public suite issue/PR links are supported.');return value;
}
export function sourceLink(source) {
  closed(source,['repository','revision','path']);
  if(!repos.has(source.repository)||typeof source.revision!=='string'||source.revision.length!==40||!/^[a-f0-9]{40}$/.test(source.revision))throw new Error('Pinned public source revision required.');
  const path=text(source.path,512),parts=path.split('/');
  if(!parts.length||parts.some(p=>!p||p==='.'||p==='..'||/[\\\u0000-\u001f\u007f]/.test(p)))throw new Error('Normalized public source path required.');
  return `https://github.com/egohygiene/${source.repository}/blob/${source.revision}/${parts.map(encodeURIComponent).join('/')}`;
}
export function workflowCatalog(value,rawSha256,locator) {
  closed(value,['schema_version','claim_scope','runtime_qualified','observed_executions','workflows']);
  if(value.schema_version!=='flow.workflow-guide-catalog/v1'||value.claim_scope!=='source_guides_only'||value.runtime_qualified!==false||value.observed_executions!==0||typeof rawSha256!=='string'||rawSha256.length!==64||!/^[a-f0-9]{64}$/.test(rawSha256)||!Array.isArray(value.workflows)||!value.workflows.length||value.workflows.length>64)throw new Error('Source guides cannot claim executed or qualified workflows.');
  const ids=new Set();const rows=value.workflows.map(w=>{
    closed(w,['id','title','description','input_output','availability','source','issue','pull_request','steps','prerequisites','remaining']);
    const id=text(w.id,128);if(id!==id.trim()||!/^workflow:[a-z0-9_-]+$/.test(id)||ids.has(id)||!['draft_source','merged_source'].includes(w.availability))throw new Error('Unique source guide identity/availability required.');ids.add(id);
    if(!Array.isArray(w.steps)||!w.steps.length||w.steps.length>16)throw new Error('Bounded workflow steps required.');
    const steps=w.steps.map(step=>{closed(step,['owner','action']);if(!repos.has(step.owner)&&step.owner!=='operator')throw new Error('Known stage owner required.');return{owner:step.owner,action:text(step.action,2048)};});
    return{id,title:text(w.title,256),description:text(w.description,2048),inputOutput:text(w.input_output,2048),availability:w.availability,sourceUrl:sourceLink(w.source),sourceRevision:w.source.revision,issueUrl:publicLink(w.issue,'issues'),pullRequest:w.pull_request===null?null:publicLink(w.pull_request,'pull'),steps,prerequisites:strings(w.prerequisites),remaining:strings(w.remaining)};
  });return{rows,rawSha256,locator:text(locator,512),claimScope:'source guides; execution not observed'};
}
