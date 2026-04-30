'use strict';

const { Contract } = require('fabric-contract-api');

class TraceabilityContract extends Contract {
  storageKey(recordId) {
    return `trace-record:${recordId}`;
  }

  async PutTraceRecord(ctx, recordJson) {
    const record = JSON.parse(recordJson);
    if (!record.record_id) {
      throw new Error('record_id is required');
    }

    const payload = record.payload || {};
    const policy = record.policy || {};
    const stored = {
      record_id: record.record_id,
      entity_id: payload.entity_id || '',
      entity_type: payload.entity_type || '',
      event_type: payload.event_type || '',
      timestamp: payload.timestamp || '',
      actor_id: payload.actor_id || '',
      location_id: payload.location_id || '',
      previous_record_ids: payload.previous_record_ids || [],
      attributes: payload.attributes || {},
      visibility: policy.visibility || 'public',
      owner_org: policy.owner_org || 'Org1MSP'
    };

    await ctx.stub.putState(this.storageKey(record.record_id), Buffer.from(JSON.stringify(stored)));
  }

  async PutTraceBatch(ctx, recordsJson) {
    const records = JSON.parse(recordsJson);
    for (const record of records) {
      await this.PutTraceRecord(ctx, JSON.stringify(record));
    }
  }

  async GetTraceRecord(ctx, recordId) {
    const bytes = await ctx.stub.getState(this.storageKey(recordId));
    if (!bytes || bytes.length === 0) {
      throw new Error(`record not found: ${recordId}`);
    }
    return bytes.toString();
  }

  async CheckPolicy(ctx, recordId, actorOrg, action) {
    const record = JSON.parse(await this.GetTraceRecord(ctx, recordId));
    switch (record.visibility) {
      case 'public':
        return true;
      case 'restricted':
        return actorOrg === record.owner_org || actorOrg === 'AuditorMSP';
      case 'private':
        return actorOrg === record.owner_org;
      default:
        throw new Error(`unknown visibility: ${record.visibility}`);
    }
  }
}

module.exports.TraceabilityContract = TraceabilityContract;
