import {describe,expect,it} from "vitest";
import {actualSizeScale,clampTransform,fillScale,fittedSize,zoomAt} from "./viewerTransform";

describe("viewer transform",()=>{
  const viewport={width:800,height:600},wide={width:1600,height:900};
  it("fits without distorting",()=>expect(fittedSize(viewport,wide)).toEqual({width:800,height:450}));
  it("anchors zoom under the cursor",()=>expect(zoomAt({scale:1,x:0,y:0},2,{x:200,y:0},viewport,wide)).toEqual({scale:2,x:-200,y:0}));
  it("never lets the image disappear beyond its edges",()=>expect(clampTransform({scale:2,x:9999,y:-9999},viewport,wide)).toEqual({scale:2,x:400,y:-150}));
  it("maps actual pixels and fill independently",()=>{expect(actualSizeScale(viewport,wide)).toBe(2);expect(fillScale(viewport,wide)).toBeCloseTo(4/3)});
  it("resets pan at fit and caps hostile scale values",()=>expect(clampTransform({scale:99,x:99,y:99},viewport,{width:0,height:0})).toEqual({scale:12,x:0,y:0}));
});
