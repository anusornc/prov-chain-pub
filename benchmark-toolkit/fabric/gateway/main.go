package main

import (
	"context"
	"crypto/x509"
	"encoding/json"
	"errors"
	"fmt"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/hyperledger/fabric-gateway/pkg/client"
	"github.com/hyperledger/fabric-gateway/pkg/identity"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
)

type config struct {
	ListenAddress string
	Channel       string
	Chaincode     string
	MSPID         string
	CryptoPath    string
	CertPath      string
	KeyDir        string
	TLSCertPath   string
	PeerEndpoint  string
	GatewayPeer   string
}

type gatewayServer struct {
	cfg      config
	contract *client.Contract
}

type recordRequest struct {
	RecordID string                 `json:"record_id"`
	Payload  map[string]interface{} `json:"payload"`
	Policy   map[string]interface{} `json:"policy"`
}

type batchRequest struct {
	Records []recordRequest `json:"records"`
}

type submitResponse struct {
	Success         bool    `json:"success"`
	TxID            string  `json:"tx_id,omitempty"`
	SubmitLatencyMS float64 `json:"submit_latency_ms"`
	CommitLatencyMS float64 `json:"commit_latency_ms"`
	BlockNumber      uint64  `json:"block_number,omitempty"`
}

type batchResponse struct {
	Success         bool    `json:"success"`
	Submitted       int     `json:"submitted"`
	Committed       int     `json:"committed"`
	SubmitLatencyMS float64 `json:"submit_latency_ms"`
	CommitLatencyMS float64 `json:"commit_latency_ms"`
}

type policyRequest struct {
	RecordID string `json:"record_id"`
	ActorOrg string `json:"actor_org"`
	Action   string `json:"action"`
}

type policyResponse struct {
	Authorized      bool    `json:"authorized"`
	PolicyLatencyMS float64 `json:"policy_latency_ms"`
}

func main() {
	cfg := loadConfig()
	ctx := context.Background()
	contract, closeGateway, err := connectGateway(ctx, cfg)
	if err != nil {
		log.Fatalf("failed to connect Fabric gateway: %v", err)
	}
	defer closeGateway()

	server := &gatewayServer{cfg: cfg, contract: contract}
	mux := http.NewServeMux()
	mux.HandleFunc("/health", server.health)
	mux.HandleFunc("/ledger/records", server.submitRecord)
	mux.HandleFunc("/ledger/records/batch", server.submitBatch)
	mux.HandleFunc("/policy/check", server.checkPolicy)

	log.Printf("Fabric benchmark gateway listening on %s channel=%s chaincode=%s", cfg.ListenAddress, cfg.Channel, cfg.Chaincode)
	if err := http.ListenAndServe(cfg.ListenAddress, mux); err != nil {
		log.Fatal(err)
	}
}

func loadConfig() config {
	cryptoPath := env("FABRIC_CRYPTO_PATH", "/fabric/organizations/peerOrganizations/org1.example.com")
	return config{
		ListenAddress: env("GATEWAY_LISTEN_ADDRESS", ":8800"),
		Channel:       env("FABRIC_CHANNEL", "provchain"),
		Chaincode:     env("FABRIC_CHAINCODE", "traceability"),
		MSPID:         env("FABRIC_MSP_ID", "Org1MSP"),
		CryptoPath:    cryptoPath,
		CertPath:      env("FABRIC_CERT_PATH", filepath.Join(cryptoPath, "users/User1@org1.example.com/msp/signcerts/cert.pem")),
		KeyDir:        env("FABRIC_KEY_DIR", filepath.Join(cryptoPath, "users/User1@org1.example.com/msp/keystore")),
		TLSCertPath:   env("FABRIC_TLS_CERT_PATH", filepath.Join(cryptoPath, "peers/peer0.org1.example.com/tls/ca.crt")),
		PeerEndpoint:  env("FABRIC_PEER_ENDPOINT", "peer0.org1.example.com:7051"),
		GatewayPeer:   env("FABRIC_GATEWAY_PEER", "peer0.org1.example.com"),
	}
}

func env(key string, fallback string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return fallback
}

func connectGateway(ctx context.Context, cfg config) (*client.Contract, func(), error) {
	clientConnection, err := newGrpcConnection(cfg)
	if err != nil {
		return nil, nil, err
	}
	id, err := newIdentity(cfg)
	if err != nil {
		_ = clientConnection.Close()
		return nil, nil, err
	}
	sign, err := newSign(cfg)
	if err != nil {
		_ = clientConnection.Close()
		return nil, nil, err
	}
	gw, err := client.Connect(
		id,
		client.WithSign(sign),
		client.WithClientConnection(clientConnection),
		client.WithEvaluateTimeout(30*time.Second),
		client.WithEndorseTimeout(30*time.Second),
		client.WithSubmitTimeout(30*time.Second),
		client.WithCommitStatusTimeout(120*time.Second),
	)
	if err != nil {
		_ = clientConnection.Close()
		return nil, nil, err
	}

	network := gw.GetNetwork(cfg.Channel)
	contract := network.GetContract(cfg.Chaincode)
	closeFn := func() {
		gw.Close()
		_ = clientConnection.Close()
	}
	_ = ctx
	return contract, closeFn, nil
}

func newGrpcConnection(cfg config) (*grpc.ClientConn, error) {
	certificatePEM, err := os.ReadFile(cfg.TLSCertPath)
	if err != nil {
		return nil, fmt.Errorf("read TLS certificate: %w", err)
	}
	certPool := x509.NewCertPool()
	if !certPool.AppendCertsFromPEM(certificatePEM) {
		return nil, errors.New("failed to add TLS certificate to cert pool")
	}
	transportCredentials := credentials.NewClientTLSFromCert(certPool, cfg.GatewayPeer)
	return grpc.Dial(cfg.PeerEndpoint, grpc.WithTransportCredentials(transportCredentials))
}

func newIdentity(cfg config) (*identity.X509Identity, error) {
	certificatePEM, err := os.ReadFile(cfg.CertPath)
	if err != nil {
		return nil, fmt.Errorf("read identity certificate: %w", err)
	}
	certificate, err := identity.CertificateFromPEM(certificatePEM)
	if err != nil {
		return nil, fmt.Errorf("parse identity certificate: %w", err)
	}
	return identity.NewX509Identity(cfg.MSPID, certificate)
}

func newSign(cfg config) (identity.Sign, error) {
	privateKeyPEM, err := os.ReadFile(firstKeyFile(cfg.KeyDir))
	if err != nil {
		return nil, fmt.Errorf("read private key: %w", err)
	}
	privateKey, err := identity.PrivateKeyFromPEM(privateKeyPEM)
	if err != nil {
		return nil, fmt.Errorf("parse private key: %w", err)
	}
	return identity.NewPrivateKeySign(privateKey)
}

func firstKeyFile(dir string) string {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return filepath.Join(dir, "missing-key")
	}
	for _, entry := range entries {
		if !entry.IsDir() && strings.HasSuffix(entry.Name(), "_sk") {
			return filepath.Join(dir, entry.Name())
		}
	}
	for _, entry := range entries {
		if !entry.IsDir() {
			return filepath.Join(dir, entry.Name())
		}
	}
	return filepath.Join(dir, "missing-key")
}

func (s *gatewayServer) health(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method not allowed"})
		return
	}
	writeJSON(w, http.StatusOK, map[string]string{
		"status":    "ok",
		"channel":   s.cfg.Channel,
		"chaincode": s.cfg.Chaincode,
	})
}

func (s *gatewayServer) submitRecord(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method not allowed"})
		return
	}
	var req recordRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
		return
	}
	if req.RecordID == "" {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "record_id is required"})
		return
	}

	recordJSON, err := json.Marshal(req)
	if err != nil {
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
		return
	}

	start := time.Now()
	txn, err := s.contract.NewProposal("PutTraceRecord", client.WithArguments(string(recordJSON)))
	if err != nil {
		writeJSON(w, http.StatusBadGateway, map[string]string{"error": err.Error()})
		return
	}
	transaction, err := txn.Endorse()
	if err != nil {
		writeJSON(w, http.StatusBadGateway, map[string]string{"error": err.Error()})
		return
	}
	submitMS := elapsedMS(start)
	commit, err := transaction.Submit()
	if err != nil {
		writeJSON(w, http.StatusBadGateway, map[string]string{"error": err.Error()})
		return
	}
	status, err := commit.Status()
	if err != nil {
		writeJSON(w, http.StatusBadGateway, map[string]string{"error": err.Error()})
		return
	}
	commitMS := elapsedMS(start)
	writeJSON(w, http.StatusOK, submitResponse{
		Success:         status.Successful,
		TxID:            transaction.TransactionID(),
		SubmitLatencyMS: submitMS,
		CommitLatencyMS: commitMS,
		BlockNumber:     status.BlockNumber,
	})
}

func (s *gatewayServer) submitBatch(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method not allowed"})
		return
	}
	var req batchRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
		return
	}
	recordsJSON, err := json.Marshal(req.Records)
	if err != nil {
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
		return
	}

	start := time.Now()
	txn, err := s.contract.NewProposal("PutTraceBatch", client.WithArguments(string(recordsJSON)))
	if err != nil {
		writeJSON(w, http.StatusBadGateway, map[string]string{"error": err.Error()})
		return
	}
	transaction, err := txn.Endorse()
	if err != nil {
		writeJSON(w, http.StatusBadGateway, map[string]string{"error": err.Error()})
		return
	}
	submitMS := elapsedMS(start)
	commit, err := transaction.Submit()
	if err != nil {
		writeJSON(w, http.StatusBadGateway, map[string]string{"error": err.Error()})
		return
	}
	status, err := commit.Status()
	if err != nil {
		writeJSON(w, http.StatusBadGateway, map[string]string{"error": err.Error()})
		return
	}
	committed := 0
	if status.Successful {
		committed = len(req.Records)
	}
	writeJSON(w, http.StatusOK, batchResponse{
		Success:         status.Successful,
		Submitted:       len(req.Records),
		Committed:       committed,
		SubmitLatencyMS: submitMS,
		CommitLatencyMS: elapsedMS(start),
	})
}

func (s *gatewayServer) checkPolicy(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method not allowed"})
		return
	}
	var req policyRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
		return
	}
	start := time.Now()
	result, err := s.contract.EvaluateTransaction("CheckPolicy", req.RecordID, req.ActorOrg, req.Action)
	if err != nil {
		writeJSON(w, http.StatusBadGateway, map[string]string{"error": err.Error()})
		return
	}
	authorized := strings.TrimSpace(string(result)) == "true"
	writeJSON(w, http.StatusOK, policyResponse{
		Authorized:      authorized,
		PolicyLatencyMS: elapsedMS(start),
	})
}

func elapsedMS(start time.Time) float64 {
	return float64(time.Since(start).Microseconds()) / 1000.0
}

func writeJSON(w http.ResponseWriter, status int, value interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	if err := json.NewEncoder(w).Encode(value); err != nil {
		log.Printf("failed to write JSON response: %v", err)
	}
}
