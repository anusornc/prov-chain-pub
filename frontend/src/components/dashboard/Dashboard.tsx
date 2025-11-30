import React, { useState, useEffect } from "react";
import {
  Activity,
  Box,
  Users,
  Database,
  TrendingUp,
  Clock,
  CheckCircle,
  AlertTriangle,
  Zap,
  Globe,
  RefreshCw,
} from "lucide-react";
import Button from "../ui/Button";
import useBlockchain from "../../hooks/useBlockchain";
import useWebSocket from "../../hooks/useWebSocket";
import type { NetworkHealth } from "../../types";
import { API_ENDPOINTS } from "../../config/api";

interface MetricCardProps {
  title: string;
  value: string | number;
  change?: string;
  changeType?: "positive" | "negative" | "neutral";
  icon: React.ReactNode;
  loading?: boolean;
}

const MetricCard: React.FC<MetricCardProps> = ({
  title,
  value,
  change,
  changeType = "neutral",
  icon,
  loading = false,
}) => {
  const changeColors = {
    positive: "text-green-600 dark:text-green-400",
    negative: "text-red-600 dark:text-red-400",
    neutral: "text-gray-600 dark:text-gray-400",
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl shadow-soft border border-gray-200 dark:border-gray-700 p-6 transition-all duration-200 hover:shadow-medium">
      <div className="flex items-center justify-between">
        <div className="flex-1">
          <p className="text-sm font-medium text-gray-600 dark:text-gray-400 mb-1">
            {title}
          </p>
          {loading ? (
            <div className="h-8 bg-gray-200 dark:bg-gray-700 rounded animate-pulse"></div>
          ) : (
            <p className="text-2xl font-bold text-gray-900 dark:text-white">
              {value}
            </p>
          )}
          {change && (
            <p className={`text-sm mt-1 ${changeColors[changeType]}`}>
              {change}
            </p>
          )}
        </div>
        <div className="ml-4 p-3 bg-primary-50 dark:bg-primary-900/20 rounded-lg">
          {icon}
        </div>
      </div>
    </div>
  );
};

interface NetworkStatusProps {
  health: NetworkHealth;
  loading?: boolean;
}

const NetworkStatus: React.FC<NetworkStatusProps> = ({
  health,
  loading = false,
}) => {
  const getStatusColor = (status: string) => {
    switch (status) {
      case "healthy":
        return "text-green-600 dark:text-green-400";
      case "warning":
        return "text-yellow-600 dark:text-yellow-400";
      case "error":
        return "text-red-600 dark:text-red-400";
      default:
        return "text-gray-600 dark:text-gray-400";
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "healthy":
        return <CheckCircle className="w-5 h-5" />;
      case "warning":
        return <AlertTriangle className="w-5 h-5" />;
      case "error":
        return <AlertTriangle className="w-5 h-5" />;
      default:
        return <Clock className="w-5 h-5" />;
    }
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl shadow-soft border border-gray-200 dark:border-gray-700 p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
          Network Status
        </h3>
        <div className={`flex items-center ${getStatusColor(health.status)}`}>
          {getStatusIcon(health.status)}
          <span className="ml-2 text-sm font-medium capitalize">
            {health.status}
          </span>
        </div>
      </div>

      {loading ? (
        <div className="space-y-3">
          {[...Array(4)].map((_, i) => (
            <div
              key={i}
              className="h-4 bg-gray-200 dark:bg-gray-700 rounded animate-pulse"
            ></div>
          ))}
        </div>
      ) : (
        <div className="space-y-3">
          <div className="flex justify-between items-center">
            <span className="text-sm text-gray-600 dark:text-gray-400">
              Uptime
            </span>
            <span className="text-sm font-medium text-gray-900 dark:text-white">
              {Math.floor(health.uptime / 3600)}h{" "}
              {Math.floor((health.uptime % 3600) / 60)}m
            </span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-sm text-gray-600 dark:text-gray-400">
              Connected Peers
            </span>
            <span className="text-sm font-medium text-gray-900 dark:text-white">
              {health.peer_count}
            </span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-sm text-gray-600 dark:text-gray-400">
              Sync Status
            </span>
            <span
              className={`text-sm font-medium capitalize ${getStatusColor(health.sync_status)}`}
            >
              {health.sync_status}
            </span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-sm text-gray-600 dark:text-gray-400">
              Last Block
            </span>
            <span className="text-sm font-medium text-gray-900 dark:text-white">
              {health.last_block_age}s ago
            </span>
          </div>
        </div>
      )}
    </div>
  );
};

const Dashboard: React.FC = () => {
  const { metrics, networkHealth, loading, error, refresh } = useBlockchain();

  const { isConnected } = useWebSocket();
  const [recentActivity, setRecentActivity] = useState<
    Array<{
      id: string;
      type: "block" | "transaction" | "participant" | "item";
      message: string;
      timestamp: string;
      color: string;
    }>
  >([]);

  // Set up real-time activity updates from backend
  useEffect(() => {
    const fetchRecentActivity = async () => {
      try {
        const response = await fetch(
          `${API_ENDPOINTS.API}/transactions/recent`,
          {
            headers: {
              Authorization: `Bearer ${localStorage.getItem("authToken")}`,
              "Content-Type": "application/json",
            },
          },
        );

        if (response.ok) {
          const data = await response.json();
          const activities = data.transactions.slice(0, 5).map(
            (
              tx: {
                id?: string;
                type?: string;
                timestamp: string;
                data?: { subject?: string };
              },
              index: number,
            ) => ({
              id: tx.id || `activity_${index}`,
              type: (tx.type === "RDF_Data" ? "transaction" : "block") as
                | "block"
                | "transaction"
                | "participant"
                | "item",
              message: `${tx.type === "RDF_Data" ? "RDF data added" : "Block created"} - ${tx.data?.subject || "Unknown"}`,
              timestamp: tx.timestamp,
              color: tx.type === "RDF_Data" ? "bg-blue-400" : "bg-green-400",
            }),
          );
          setRecentActivity(activities);
        } else {
          // Fallback to basic activity based on metrics
          const activities: Array<{
            id: string;
            type: "block" | "transaction" | "participant" | "item";
            message: string;
            timestamp: string;
            color: string;
          }> = [];
          if (metrics?.total_blocks && metrics.total_blocks > 0) {
            activities.push({
              id: "latest_block",
              type: "block",
              message: `Latest block #${metrics.total_blocks - 1} created`,
              timestamp: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
              color: "bg-green-400",
            });
          }
          if (metrics?.total_transactions && metrics.total_transactions > 0) {
            activities.push({
              id: "latest_transaction",
              type: "transaction",
              message: `${metrics.total_transactions} transactions processed`,
              timestamp: new Date(Date.now() - 10 * 60 * 1000).toISOString(),
              color: "bg-blue-400",
            });
          }
          setRecentActivity(activities);
        }
      } catch (error) {
        console.error("Error fetching recent activity:", error);
        // Fallback to metrics-based activity
        const activities: Array<{
          id: string;
          type: "block" | "transaction" | "participant" | "item";
          message: string;
          timestamp: string;
          color: string;
        }> = [];
        if (metrics?.total_blocks && metrics.total_blocks > 0) {
          activities.push({
            id: "system_status",
            type: "block",
            message: `System active - ${metrics.total_blocks} blocks processed`,
            timestamp: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
            color: "bg-green-400",
          });
        }
        setRecentActivity(activities);
      }
    };

    fetchRecentActivity();

    // Refresh activity every 30 seconds
    const interval = setInterval(fetchRecentActivity, 30000);
    return () => clearInterval(interval);
  }, [metrics?.total_blocks, metrics?.total_transactions]);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                Blockchain Explorer Dashboard
              </h1>
              <p className="text-gray-600 dark:text-gray-400">
                Real-time overview of the ProvChain network and traceability
                system
              </p>
            </div>
            <div className="flex items-center space-x-4">
              <div
                className={`flex items-center space-x-2 ${isConnected ? "text-green-600 dark:text-green-400" : "text-red-600 dark:text-red-400"}`}
              >
                <div
                  className={`w-2 h-2 rounded-full ${isConnected ? "bg-green-400" : "bg-red-400"}`}
                ></div>
                <span className="text-sm font-medium">
                  {isConnected ? "Connected" : "Disconnected"}
                </span>
              </div>
              <Button variant="outline" onClick={refresh} disabled={loading}>
                <RefreshCw
                  className={`w-4 h-4 mr-2 ${loading ? "animate-spin" : ""}`}
                />
                Refresh
              </Button>
            </div>
          </div>
        </div>

        {/* Error State */}
        {error && (
          <div className="mb-8 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
            <div className="flex items-center space-x-2 text-red-600 dark:text-red-400">
              <AlertTriangle className="w-5 h-5" />
              <span>Error loading dashboard data: {error}</span>
            </div>
          </div>
        )}

        {/* Metrics Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <MetricCard
            title="Total Blocks"
            value={metrics?.total_blocks.toLocaleString() || "0"}
            change={
              error ? "Network Error" : isConnected ? "Live Data" : "Offline"
            }
            changeType={
              error ? "negative" : isConnected ? "positive" : "neutral"
            }
            icon={
              <Box className="w-6 h-6 text-primary-600 dark:text-primary-400" />
            }
            loading={loading}
          />
          <MetricCard
            title="Total Transactions"
            value={metrics?.total_transactions.toLocaleString() || "0"}
            change={
              error ? "Network Error" : isConnected ? "Live Data" : "Offline"
            }
            changeType={
              error ? "negative" : isConnected ? "positive" : "neutral"
            }
            icon={
              <Activity className="w-6 h-6 text-primary-600 dark:text-primary-400" />
            }
            loading={loading}
          />
          <MetricCard
            title="Traced Items"
            value={metrics?.total_items.toLocaleString() || "0"}
            change={
              error ? "Network Error" : isConnected ? "Live Data" : "Offline"
            }
            changeType={
              error ? "negative" : isConnected ? "positive" : "neutral"
            }
            icon={
              <Database className="w-6 h-6 text-primary-600 dark:text-primary-400" />
            }
            loading={loading}
          />
          <MetricCard
            title="Active Participants"
            value={metrics?.active_participants || "0"}
            change={
              error ? "Network Error" : isConnected ? "Live Data" : "Offline"
            }
            changeType={
              error ? "negative" : isConnected ? "positive" : "neutral"
            }
            icon={
              <Users className="w-6 h-6 text-primary-600 dark:text-primary-400" />
            }
            loading={loading}
          />
        </div>

        {/* Performance Metrics */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <MetricCard
            title="Avg Block Time"
            value={`${metrics?.avg_block_time || "0"}s`}
            change={
              error ? "Network Error" : isConnected ? "Live Data" : "Offline"
            }
            changeType={
              error ? "negative" : isConnected ? "positive" : "neutral"
            }
            icon={
              <Clock className="w-6 h-6 text-primary-600 dark:text-primary-400" />
            }
            loading={loading}
          />
          <MetricCard
            title="TPS"
            value={metrics?.transactions_per_second || "0"}
            change={
              error ? "Network Error" : isConnected ? "Live Data" : "Offline"
            }
            changeType={
              error ? "negative" : isConnected ? "positive" : "neutral"
            }
            icon={
              <Zap className="w-6 h-6 text-primary-600 dark:text-primary-400" />
            }
            loading={loading}
          />
          <MetricCard
            title="Network Hash Rate"
            value={`${((metrics?.network_hash_rate || 0) / 1000000).toFixed(1)}M`}
            change={
              error ? "Network Error" : isConnected ? "Live Data" : "Offline"
            }
            changeType={
              error ? "negative" : isConnected ? "positive" : "neutral"
            }
            icon={
              <TrendingUp className="w-6 h-6 text-primary-600 dark:text-primary-400" />
            }
            loading={loading}
          />
          <MetricCard
            title="Network Status"
            value={metrics?.network_status || "Unknown"}
            icon={
              <Globe className="w-6 h-6 text-primary-600 dark:text-primary-400" />
            }
            loading={loading}
          />
        </div>

        {/* Network Status and Recent Activity */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
          {networkHealth && (
            <NetworkStatus health={networkHealth} loading={loading} />
          )}

          <div className="bg-white dark:bg-gray-800 rounded-xl shadow-soft border border-gray-200 dark:border-gray-700 p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              Recent Activity
            </h3>
            {loading ? (
              <div className="space-y-3">
                {[...Array(5)].map((_, i) => (
                  <div
                    key={i}
                    className="h-12 bg-gray-200 dark:bg-gray-700 rounded animate-pulse"
                  ></div>
                ))}
              </div>
            ) : (
              <div className="space-y-3">
                {recentActivity.map((activity) => (
                  <div
                    key={activity.id}
                    className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg"
                  >
                    <div className="flex items-center">
                      <div
                        className={`w-2 h-2 ${activity.color} rounded-full mr-3`}
                      ></div>
                      <span className="text-sm text-gray-900 dark:text-white">
                        {activity.message}
                      </span>
                    </div>
                    <span className="text-xs text-gray-500 dark:text-gray-400">
                      {new Date(activity.timestamp).toLocaleTimeString()}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default Dashboard;
