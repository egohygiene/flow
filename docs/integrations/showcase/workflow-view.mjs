// Local text-node-only rendering; links are validated by the source-guide model.
import {workflowCatalog} from './workflows.mjs';
let current=null;
const $=id=>document.getElementById(id);
function node(tag,text,parent){const n=document.createElement(tag);if(text!==undefined)n.textContent=text;parent.append(n);return n;}
function link(label,url,parent){const n=node('a',label,parent);n.href=url;n.rel='noreferrer';return n;}
function list(title,values,parent){const d=node('details',undefined,parent);node('summary',title,d);const ul=node('ul',undefined,d);for(const v of values)node('li',v,ul);}
export function loadWorkflows(value,digest,locator){const parsed=workflowCatalog(value,digest,locator);current=parsed;renderWorkflows();}
export function clearWorkflows(){current=null;$('workflow-search').value='';renderWorkflows();}
function renderWorkflows(){
  const root=$('workflows');root.replaceChildren();const q=$('workflow-search').value.trim().toLowerCase();const rows=(current?.rows??[]).filter(w=>!q||JSON.stringify(w).toLowerCase().includes(q));
  $('workflow-count').textContent=current?`${rows.length} of ${current.rows.length} source guides. These are not test scenarios or observed successful runs. Catalog SHA-256: ${current.rawSha256}`:'No workflow guides loaded. Execution evidence remains unavailable.';
  for(const w of rows){const card=node('article',undefined,root);card.id=w.id;node('h3',w.title,card);node('p',`${w.availability}: source snapshot only; native/release qualification not established`,card);node('p',w.description,card);node('p',w.inputOutput,card);
    const stages=node('ol',undefined,card);for(const step of w.steps)node('li',`${step.owner}: ${step.action}`,stages);
    list('Prerequisites',w.prerequisites,card);list('Remaining work and review gates',w.remaining,card);const refs=node('p',undefined,card);link('Owning issue',w.issueUrl,refs);node('span',' · ',refs);link('Pinned source',w.sourceUrl,refs);if(w.pullRequest){node('span',' · ',refs);link('Draft checkpoint',w.pullRequest,refs);}node('p',`Source revision: ${w.sourceRevision}`,card).className='identity';
  }
}
$('workflow-search').addEventListener('input',renderWorkflows);
