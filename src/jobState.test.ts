import { describe,expect,it } from "vitest";
import { isJobPollingFast,jobBucket,jobNextStep } from "./jobState";
import type { JobOverview } from "./types";

const job=(state:string):JobOverview=>({jobId:"j",sourceName:"Fonte",sourcePath:"E:\\DCIM",state,stage:state,processedItems:10,totalItems:10,processedBytes:100,totalBytes:100,overallPercent:100,imported:10,duplicates:0,excluded:0,failed:0,createdAt:"2026-01-01T00:00:00Z",updatedAt:"2026-01-01T00:01:00Z"});

describe("ciclo operacional dos jobs",()=>{
  it("não trata estados pós-importação como processamento preso",()=>{
    expect(jobBucket("protection_pending")).toBe("attention");
    expect(jobBucket("batch_pending")).toBe("attention");
    expect(isJobPollingFast([job("protection_pending")])).toBe(false);
    expect(jobNextStep(job("protection_pending"))).toContain("já estão no acervo");
  });
  it("mantém trabalho executável ativo e finais no histórico",()=>{
    expect(jobBucket("protecting")).toBe("active");
    expect(jobBucket("completed")).toBe("history");
    expect(isJobPollingFast([job("protecting")])).toBe(true);
  });
});
