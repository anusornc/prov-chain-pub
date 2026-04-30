package main

import (
	"encoding/json"
	"fmt"

	"github.com/hyperledger/fabric-contract-api-go/contractapi"
)

type TraceabilityContract struct {
	contractapi.Contract
}

type GatewayRecord struct {
	RecordID string        `json:"record_id"`
	Payload  RecordPayload `json:"payload"`
	Policy   RecordPolicy  `json:"policy"`
}

type RecordPayload struct {
	EntityID          string                 `json:"entity_id"`
	EntityType        string                 `json:"entity_type"`
	EventType         string                 `json:"event_type"`
	Timestamp         string                 `json:"timestamp"`
	ActorID           string                 `json:"actor_id"`
	LocationID        string                 `json:"location_id,omitempty"`
	PreviousRecordIDs []string               `json:"previous_record_ids,omitempty"`
	Attributes        map[string]interface{} `json:"attributes,omitempty"`
}

type RecordPolicy struct {
	Visibility string `json:"visibility"`
	OwnerOrg   string `json:"owner_org"`
}

type StoredRecord struct {
	RecordID          string                 `json:"record_id"`
	EntityID          string                 `json:"entity_id"`
	EntityType        string                 `json:"entity_type"`
	EventType         string                 `json:"event_type"`
	Timestamp         string                 `json:"timestamp"`
	ActorID           string                 `json:"actor_id"`
	LocationID        string                 `json:"location_id,omitempty"`
	PreviousRecordIDs []string               `json:"previous_record_ids,omitempty"`
	Attributes        map[string]interface{} `json:"attributes,omitempty"`
	Visibility        string                 `json:"visibility"`
	OwnerOrg          string                 `json:"owner_org"`
}

func storageKey(recordID string) string {
	return "trace-record:" + recordID
}

func (c *TraceabilityContract) PutTraceRecord(ctx contractapi.TransactionContextInterface, recordJSON string) error {
	var record GatewayRecord
	if err := json.Unmarshal([]byte(recordJSON), &record); err != nil {
		return fmt.Errorf("invalid record JSON: %w", err)
	}
	if record.RecordID == "" {
		return fmt.Errorf("record_id is required")
	}
	if record.Policy.Visibility == "" {
		record.Policy.Visibility = "public"
	}
	if record.Policy.OwnerOrg == "" {
		record.Policy.OwnerOrg = "Org1MSP"
	}

	stored := StoredRecord{
		RecordID:          record.RecordID,
		EntityID:          record.Payload.EntityID,
		EntityType:        record.Payload.EntityType,
		EventType:         record.Payload.EventType,
		Timestamp:         record.Payload.Timestamp,
		ActorID:           record.Payload.ActorID,
		LocationID:        record.Payload.LocationID,
		PreviousRecordIDs: record.Payload.PreviousRecordIDs,
		Attributes:        record.Payload.Attributes,
		Visibility:        record.Policy.Visibility,
		OwnerOrg:          record.Policy.OwnerOrg,
	}

	bytes, err := json.Marshal(stored)
	if err != nil {
		return fmt.Errorf("failed to marshal record: %w", err)
	}
	return ctx.GetStub().PutState(storageKey(record.RecordID), bytes)
}

func (c *TraceabilityContract) PutTraceBatch(ctx contractapi.TransactionContextInterface, recordsJSON string) error {
	var records []GatewayRecord
	if err := json.Unmarshal([]byte(recordsJSON), &records); err != nil {
		return fmt.Errorf("invalid records JSON: %w", err)
	}
	for _, record := range records {
		bytes, err := json.Marshal(record)
		if err != nil {
			return fmt.Errorf("failed to marshal batch record %s: %w", record.RecordID, err)
		}
		if err := c.PutTraceRecord(ctx, string(bytes)); err != nil {
			return err
		}
	}
	return nil
}

func (c *TraceabilityContract) GetTraceRecord(ctx contractapi.TransactionContextInterface, recordID string) (*StoredRecord, error) {
	bytes, err := ctx.GetStub().GetState(storageKey(recordID))
	if err != nil {
		return nil, fmt.Errorf("failed to read record %s: %w", recordID, err)
	}
	if len(bytes) == 0 {
		return nil, fmt.Errorf("record not found: %s", recordID)
	}
	var record StoredRecord
	if err := json.Unmarshal(bytes, &record); err != nil {
		return nil, fmt.Errorf("failed to decode record %s: %w", recordID, err)
	}
	return &record, nil
}

func (c *TraceabilityContract) CheckPolicy(ctx contractapi.TransactionContextInterface, recordID string, actorOrg string, action string) (bool, error) {
	record, err := c.GetTraceRecord(ctx, recordID)
	if err != nil {
		return false, err
	}
	switch record.Visibility {
	case "public":
		return true, nil
	case "restricted":
		return actorOrg == record.OwnerOrg || actorOrg == "AuditorMSP", nil
	case "private":
		return actorOrg == record.OwnerOrg, nil
	default:
		return false, fmt.Errorf("unknown visibility: %s", record.Visibility)
	}
}

func main() {
	chaincode, err := contractapi.NewChaincode(&TraceabilityContract{})
	if err != nil {
		panic(fmt.Sprintf("failed to create chaincode: %s", err))
	}
	if err := chaincode.Start(); err != nil {
		panic(fmt.Sprintf("failed to start chaincode: %s", err))
	}
}
