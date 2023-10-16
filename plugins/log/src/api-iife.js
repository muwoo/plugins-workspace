if("__TAURI__"in window){var __TAURI_LOG__=function(e){"use strict";var n=Object.defineProperty,t=(e,t)=>{for(var r in t)n(e,r,{get:t[r],enumerable:!0})},r=(e,n,t)=>{if(!n.has(e))throw TypeError("Cannot "+t)},a=(e,n,t)=>(r(e,n,"read from private field"),t?t.call(e):n.get(e)),i=(e,n,t,a)=>(r(e,n,"write to private field"),a?a.call(e,t):n.set(e,t),t);function o(e,n=!1){let t=window.crypto.getRandomValues(new Uint32Array(1))[0],r=`_${t}`;return Object.defineProperty(window,r,{value:t=>(n&&Reflect.deleteProperty(window,r),e?.(t)),writable:!1,configurable:!0}),t}t({},{Channel:()=>c,PluginListener:()=>s,addPluginListener:()=>u,convertFileSrc:()=>d,invoke:()=>_,transformCallback:()=>o});var l,c=class{constructor(){this.__TAURI_CHANNEL_MARKER__=!0,((e,n,t)=>{if(n.has(e))throw TypeError("Cannot add the same private member more than once");n instanceof WeakSet?n.add(e):n.set(e,t)})(this,l,(()=>{})),this.id=o((e=>{a(this,l).call(this,e)}))}set onmessage(e){i(this,l,e)}get onmessage(){return a(this,l)}toJSON(){return`__CHANNEL__:${this.id}`}};l=new WeakMap;var s=class{constructor(e,n,t){this.plugin=e,this.event=n,this.channelId=t}async unregister(){return _(`plugin:${this.plugin}|remove_listener`,{event:this.event,channelId:this.channelId})}};async function u(e,n,t){let r=new c;return r.onmessage=t,_(`plugin:${e}|register_listener`,{event:n,handler:r}).then((()=>new s(e,n,r.id)))}async function _(e,n={},t){return new Promise(((r,a)=>{let i=o((e=>{r(e),Reflect.deleteProperty(window,`_${l}`)}),!0),l=o((e=>{a(e),Reflect.deleteProperty(window,`_${i}`)}),!0);window.__TAURI_IPC__({cmd:e,callback:i,error:l,payload:n,options:t})}))}function d(e,n="asset"){return window.__TAURI__.convertFileSrc(e,n)}t({},{TauriEvent:()=>f,emit:()=>E,listen:()=>v,once:()=>h});var w,f=(e=>(e.WINDOW_RESIZED="tauri://resize",e.WINDOW_MOVED="tauri://move",e.WINDOW_CLOSE_REQUESTED="tauri://close-requested",e.WINDOW_CREATED="tauri://window-created",e.WINDOW_DESTROYED="tauri://destroyed",e.WINDOW_FOCUS="tauri://focus",e.WINDOW_BLUR="tauri://blur",e.WINDOW_SCALE_FACTOR_CHANGED="tauri://scale-change",e.WINDOW_THEME_CHANGED="tauri://theme-changed",e.WINDOW_FILE_DROP="tauri://file-drop",e.WINDOW_FILE_DROP_HOVER="tauri://file-drop-hover",e.WINDOW_FILE_DROP_CANCELLED="tauri://file-drop-cancelled",e.MENU="tauri://menu",e))(f||{});async function g(e,n){await _("plugin:event|unlisten",{event:e,eventId:n})}async function v(e,n,t){return _("plugin:event|listen",{event:e,windowLabel:t?.target,handler:o(n)}).then((n=>async()=>g(e,n)))}async function h(e,n,t){return v(e,(t=>{n(t),g(e,t.id).catch((()=>{}))}),t)}async function E(e,n,t){await _("plugin:event|emit",{event:e,windowLabel:t?.target,payload:n})}async function I(e,n,t){var r,a;const i=null===(r=(new Error).stack)||void 0===r?void 0:r.split("\n").map((e=>e.split("@"))),o=null==i?void 0:i.filter((([e,n])=>e.length>0&&"[native code]"!==n)),{file:l,line:c,keyValues:s}=null!=t?t:{};let u=null===(a=null==o?void 0:o[0])||void 0===a?void 0:a.filter((e=>e.length>0)).join("@");"Error"===u&&(u="webview::unknown"),await window.__TAURI_INVOKE__("plugin:log|log",{level:e,message:n,location:u,file:l,line:c,keyValues:s})}return function(e){e[e.Trace=1]="Trace",e[e.Debug=2]="Debug",e[e.Info=3]="Info",e[e.Warn=4]="Warn",e[e.Error=5]="Error"}(w||(w={})),e.attachConsole=async function(){return await v("log://log",(e=>{const n=e.payload,t=n.message.replace(/[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g,"");switch(n.level){case w.Trace:console.log(t);break;case w.Debug:console.debug(t);break;case w.Info:console.info(t);break;case w.Warn:console.warn(t);break;case w.Error:console.error(t);break;default:throw new Error(`unknown log level ${n.level}`)}}))},e.debug=async function(e,n){await I(w.Debug,e,n)},e.error=async function(e,n){await I(w.Error,e,n)},e.info=async function(e,n){await I(w.Info,e,n)},e.trace=async function(e,n){await I(w.Trace,e,n)},e.warn=async function(e,n){await I(w.Warn,e,n)},e}({});Object.defineProperty(window.__TAURI__,"log",{value:__TAURI_LOG__})}