export interface ViewerSize { width:number; height:number }
export interface ViewerPoint { x:number; y:number }
export interface ViewerTransform extends ViewerPoint { scale:number }

export function fittedSize(viewport:ViewerSize,natural:ViewerSize):ViewerSize {
  if(viewport.width<=0||viewport.height<=0||natural.width<=0||natural.height<=0)return{width:0,height:0};
  const ratio=Math.min(viewport.width/natural.width,viewport.height/natural.height);
  return{width:natural.width*ratio,height:natural.height*ratio};
}

export function clampTransform(transform:ViewerTransform,viewport:ViewerSize,natural:ViewerSize):ViewerTransform {
  const scale=Math.min(12,Math.max(1,transform.scale));
  const fit=fittedSize(viewport,natural);
  const maxX=Math.max(0,(fit.width*scale-viewport.width)/2);
  const maxY=Math.max(0,(fit.height*scale-viewport.height)/2);
  return{scale,x:Math.max(-maxX,Math.min(maxX,transform.x)),y:Math.max(-maxY,Math.min(maxY,transform.y))};
}

export function zoomAt(current:ViewerTransform,nextScale:number,anchor:ViewerPoint,viewport:ViewerSize,natural:ViewerSize):ViewerTransform {
  const scale=Math.min(12,Math.max(1,Math.round(nextScale*4)/4));
  const ratio=scale/current.scale;
  return clampTransform({scale,x:anchor.x-(anchor.x-current.x)*ratio,y:anchor.y-(anchor.y-current.y)*ratio},viewport,natural);
}

export function actualSizeScale(viewport:ViewerSize,natural:ViewerSize):number {
  const fit=fittedSize(viewport,natural);
  return fit.width?Math.min(12,Math.max(1,natural.width/fit.width)):1;
}

export function fillScale(viewport:ViewerSize,natural:ViewerSize):number {
  const fit=fittedSize(viewport,natural);
  if(!fit.width||!fit.height)return 1;
  return Math.min(12,Math.max(1,Math.max(viewport.width/fit.width,viewport.height/fit.height)));
}
