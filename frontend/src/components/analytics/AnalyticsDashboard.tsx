import React, { useState, useEffect, useCallback } from "react";
import {
  BarChart3,
  TrendingUp,
  Users,
  Activity,
  Package,
  Filter,
  Download,
  RefreshCw,
  PieChart,
  LineChart,
  AlertTriangle,
} from "lucide-react";
import LoadingSpinner from "../ui/LoadingSpinner";
import Card from "../ui/Card";
import Button from "../ui/Button";
import Input from "../ui/Input";
import type { DashboardMetrics } from "../../types";
import { API_ENDPOINTS } from "../../config/api";

interface AnalyticsData {
  metrics: DashboardMetrics;
  transactionTrends: Array<{ date: string; count: number; volume: number }>;
  participantActivity: Array<{
    participant: string;
    transactions: number;
    lastActive: string;
  }>;
  blockchainGrowth: Array<{ date: string; blocks: number; size: number }>;
  traceabilityStats: Array<{ type: string; count: number; percentage: number }>;
  performanceMetrics: Array<{
    metric: string;
    value: number;
    unit: string;
    trend: "up" | "down" | "stable";
  }>;
}

interface AnalyticsFilters {
  dateRange: {
    start: string;
    end: string;
  };
  participantType: string;
  transactionType: string;
  timeGranularity: "hour" | "day" | "week" | "month";
}

const AnalyticsDashboard: React.FC = () => {
  const [analyticsData, setAnalyticsData] = useState<AnalyticsData | null>(
    null,
  );
  const [isLoading, setIsLoading] = useState(true);
  const [filters, setFilters] = useState<AnalyticsFilters>({
    dateRange: {
      start: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000)
        .toISOString()
        .split("T")[0], // 30 days ago
      end: new Date().toISOString().split("T")[0], // today
    },
    participantType: "all",
    transactionType: "all",
    timeGranularity: "day",
  });
  const [isFiltersOpen, setIsFiltersOpen] = useState(false);

  useEffect(() => {
    loadAnalyticsData();
  }, [filters, loadAnalyticsData]);

  const loadAnalyticsData = useCallback(async () => {
    setIsLoading(true);
    try {
      const token = localStorage.getItem("authToken");

      // Try to load real analytics data from API
      try {
        const response = await fetch(
          `${API_ENDPOINTS.API}/analytics?${new URLSearchParams({
            start_date: filters.dateRange.start,
            end_date: filters.dateRange.end,
            participant_type: filters.participantType,
            transaction_type: filters.transactionType,
            granularity: filters.timeGranularity,
          })}`,
          {
            headers: {
              Authorization: `Bearer ${token}`,
              "Content-Type": "application/json",
            },
          },
        );

        if (response.ok) {
          const data: AnalyticsData = await response.json();
          setAnalyticsData(data);
        } else {
          // Fallback to mock data
          loadMockAnalyticsData();
        }
      } catch (error) {
        console.warn("Analytics API not available, using mock data:", error);
        loadMockAnalyticsData();
      }
    } catch (error) {
      console.error("Error loading analytics data:", error);
      loadMockAnalyticsData();
    } finally {
      setIsLoading(false);
    }
  }, [filters, loadMockAnalyticsData]);

  const loadMockAnalyticsData = () => {
    const mockData: AnalyticsData = {
      metrics: {
        total_blocks: 1247,
        total_transactions: 8934,
        total_items: 3456,
        active_participants: 23,
        network_status: "healthy",
        last_block_time: new Date(Date.now() - 2 * 60 * 1000).toISOString(), // 2 minutes ago
        avg_block_time: 12.5,
        transactions_per_second: 2.3,
        network_hash_rate: 1250000,
      },
      transactionTrends: generateMockTrendData(30),
      participantActivity: [
        {
          participant: "Organic Farms Co.",
          transactions: 234,
          lastActive: "2 hours ago",
        },
        {
          participant: "Global Manufacturing Ltd.",
          transactions: 189,
          lastActive: "1 hour ago",
        },
        {
          participant: "Swift Logistics",
          transactions: 156,
          lastActive: "30 minutes ago",
        },
        {
          participant: "Quality Assurance Labs",
          transactions: 143,
          lastActive: "45 minutes ago",
        },
        {
          participant: "Retail Chain Corp",
          transactions: 98,
          lastActive: "3 hours ago",
        },
      ],
      blockchainGrowth: generateMockGrowthData(30),
      traceabilityStats: [
        { type: "Production", count: 1234, percentage: 35.7 },
        { type: "Processing", count: 987, percentage: 28.5 },
        { type: "Transport", count: 654, percentage: 18.9 },
        { type: "Quality", count: 432, percentage: 12.5 },
        { type: "Other", count: 149, percentage: 4.4 },
      ],
      performanceMetrics: [
        {
          metric: "Average Block Time",
          value: 12.5,
          unit: "seconds",
          trend: "stable",
        },
        {
          metric: "Transaction Throughput",
          value: 2.3,
          unit: "TPS",
          trend: "up",
        },
        { metric: "Network Hash Rate", value: 1.25, unit: "MH/s", trend: "up" },
        { metric: "Storage Usage", value: 2.4, unit: "GB", trend: "up" },
        { metric: "Active Nodes", value: 12, unit: "nodes", trend: "stable" },
        { metric: "Validation Time", value: 45, unit: "ms", trend: "down" },
      ],
    };
    setAnalyticsData(mockData);
  };

  const generateMockTrendData = (days: number) => {
    const data = [];
    for (let i = days - 1; i >= 0; i--) {
      const date = new Date(Date.now() - i * 24 * 60 * 60 * 1000);
      data.push({
        date: date.toISOString().split("T")[0],
        count: Math.floor(Math.random() * 50) + 20,
        volume: Math.floor(Math.random() * 1000) + 500,
      });
    }
    return data;
  };

  const generateMockGrowthData = (days: number) => {
    const data = [];
    let cumulativeBlocks = 1000;
    let cumulativeSize = 1.5;

    for (let i = days - 1; i >= 0; i--) {
      const date = new Date(Date.now() - i * 24 * 60 * 60 * 1000);
      const dailyBlocks = Math.floor(Math.random() * 20) + 5;
      const dailySize = Math.random() * 0.1 + 0.05;

      cumulativeBlocks += dailyBlocks;
      cumulativeSize += dailySize;

      data.push({
        date: date.toISOString().split("T")[0],
        blocks: cumulativeBlocks,
        size: parseFloat(cumulativeSize.toFixed(2)),
      });
    }
    return data;
  };

  const handleFilterChange = (
    key: keyof AnalyticsFilters,
    value: string | { start: string; end: string },
  ) => {
    setFilters((prev) => ({ ...prev, [key]: value }));
  };

  const exportData = () => {
    if (!analyticsData) return;

    const dataToExport = {
      metrics: analyticsData.metrics,
      transactionTrends: analyticsData.transactionTrends,
      participantActivity: analyticsData.participantActivity,
      exportedAt: new Date().toISOString(),
      filters: filters,
    };

    const blob = new Blob([JSON.stringify(dataToExport, null, 2)], {
      type: "application/json",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `provchain-analytics-${new Date().toISOString().split("T")[0]}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const formatNumber = (num: number): string => {
    if (num >= 1000000) return (num / 1000000).toFixed(1) + "M";
    if (num >= 1000) return (num / 1000).toFixed(1) + "K";
    return num.toString();
  };

  const getTrendIcon = (trend: "up" | "down" | "stable") => {
    switch (trend) {
      case "up":
        return <TrendingUp className="w-4 h-4 text-green-500" />;
      case "down":
        return <TrendingUp className="w-4 h-4 text-red-500 rotate-180" />;
      case "stable":
        return <Activity className="w-4 h-4 text-gray-500" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case "healthy":
        return "text-green-500";
      case "warning":
        return "text-yellow-500";
      case "error":
        return "text-red-500";
      default:
        return "text-gray-500";
    }
  };

  if (isLoading) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center">
        <LoadingSpinner size="lg" message="Loading analytics..." />
      </div>
    );
  }

  if (!analyticsData) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
        <div className="max-w-7xl mx-auto">
          <div className="text-center py-12">
            <AlertTriangle className="w-12 h-12 text-red-500 mx-auto mb-4" />
            <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
              Failed to Load Analytics
            </h2>
            <p className="text-gray-600 dark:text-gray-300 mb-4">
              Unable to load analytics data. Please try again.
            </p>
            <Button onClick={loadAnalyticsData}>
              <RefreshCw className="w-4 h-4 mr-2" />
              Retry
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                Analytics Dashboard
              </h1>
              <p className="text-gray-600 dark:text-gray-300">
                Comprehensive blockchain analytics and performance metrics
              </p>
            </div>
            <div className="flex gap-2">
              <Button
                variant="outline"
                onClick={() => setIsFiltersOpen(!isFiltersOpen)}
                className="flex items-center gap-2"
              >
                <Filter className="w-4 h-4" />
                Filters
              </Button>
              <Button
                variant="outline"
                onClick={exportData}
                className="flex items-center gap-2"
              >
                <Download className="w-4 h-4" />
                Export
              </Button>
              <Button
                variant="outline"
                onClick={loadAnalyticsData}
                className="flex items-center gap-2"
              >
                <RefreshCw className="w-4 h-4" />
                Refresh
              </Button>
            </div>
          </div>
        </div>

        {/* Filters */}
        {isFiltersOpen && (
          <Card className="mb-6">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Date Range
                </label>
                <div className="flex gap-2">
                  <Input
                    type="date"
                    value={filters.dateRange.start}
                    onChange={(e) =>
                      handleFilterChange("dateRange", {
                        ...filters.dateRange,
                        start: e.target.value,
                      })
                    }
                    className="text-sm"
                  />
                  <Input
                    type="date"
                    value={filters.dateRange.end}
                    onChange={(e) =>
                      handleFilterChange("dateRange", {
                        ...filters.dateRange,
                        end: e.target.value,
                      })
                    }
                    className="text-sm"
                  />
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Participant Type
                </label>
                <select
                  value={filters.participantType}
                  onChange={(e) =>
                    handleFilterChange("participantType", e.target.value)
                  }
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                >
                  <option value="all">All Types</option>
                  <option value="Producer">Producer</option>
                  <option value="Manufacturer">Manufacturer</option>
                  <option value="LogisticsProvider">Logistics Provider</option>
                  <option value="QualityLab">Quality Lab</option>
                  <option value="Auditor">Auditor</option>
                  <option value="Retailer">Retailer</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Transaction Type
                </label>
                <select
                  value={filters.transactionType}
                  onChange={(e) =>
                    handleFilterChange("transactionType", e.target.value)
                  }
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                >
                  <option value="all">All Types</option>
                  <option value="Production">Production</option>
                  <option value="Processing">Processing</option>
                  <option value="Transport">Transport</option>
                  <option value="Quality">Quality</option>
                  <option value="Transfer">Transfer</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Time Granularity
                </label>
                <select
                  value={filters.timeGranularity}
                  onChange={(e) =>
                    handleFilterChange(
                      "timeGranularity",
                      e.target.value as "hour" | "day" | "week" | "month",
                    )
                  }
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                >
                  <option value="hour">Hourly</option>
                  <option value="day">Daily</option>
                  <option value="week">Weekly</option>
                  <option value="month">Monthly</option>
                </select>
              </div>
            </div>
          </Card>
        )}

        {/* Key Metrics */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <Card>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600 dark:text-gray-400">
                  Total Blocks
                </p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white">
                  {formatNumber(analyticsData.metrics.total_blocks)}
                </p>
              </div>
              <div className="p-3 bg-blue-100 dark:bg-blue-900/30 rounded-lg">
                <Package className="w-6 h-6 text-blue-600 dark:text-blue-400" />
              </div>
            </div>
          </Card>

          <Card>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600 dark:text-gray-400">
                  Total Transactions
                </p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white">
                  {formatNumber(analyticsData.metrics.total_transactions)}
                </p>
              </div>
              <div className="p-3 bg-green-100 dark:bg-green-900/30 rounded-lg">
                <Activity className="w-6 h-6 text-green-600 dark:text-green-400" />
              </div>
            </div>
          </Card>

          <Card>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600 dark:text-gray-400">
                  Active Participants
                </p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white">
                  {analyticsData.metrics.active_participants}
                </p>
              </div>
              <div className="p-3 bg-purple-100 dark:bg-purple-900/30 rounded-lg">
                <Users className="w-6 h-6 text-purple-600 dark:text-purple-400" />
              </div>
            </div>
          </Card>

          <Card>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600 dark:text-gray-400">
                  Network Status
                </p>
                <div className="flex items-center gap-2">
                  <p
                    className={`text-2xl font-bold capitalize ${getStatusColor(analyticsData.metrics.network_status)}`}
                  >
                    {analyticsData.metrics.network_status}
                  </p>
                  <div
                    className={`w-3 h-3 rounded-full ${analyticsData.metrics.network_status === "healthy" ? "bg-green-500" : analyticsData.metrics.network_status === "warning" ? "bg-yellow-500" : "bg-red-500"}`}
                  ></div>
                </div>
              </div>
              <div className="p-3 bg-orange-100 dark:bg-orange-900/30 rounded-lg">
                <BarChart3 className="w-6 h-6 text-orange-600 dark:text-orange-400" />
              </div>
            </div>
          </Card>
        </div>

        {/* Performance Metrics */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
          <Card>
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                Performance Metrics
              </h3>
              <PieChart className="w-5 h-5 text-gray-500" />
            </div>
            <div className="space-y-4">
              {analyticsData.performanceMetrics.map((metric, index) => (
                <div key={index} className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    {getTrendIcon(metric.trend)}
                    <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                      {metric.metric}
                    </span>
                  </div>
                  <div className="text-right">
                    <span className="text-lg font-semibold text-gray-900 dark:text-white">
                      {metric.value}
                    </span>
                    <span className="text-sm text-gray-500 ml-1">
                      {metric.unit}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </Card>

          <Card>
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                Traceability Distribution
              </h3>
              <PieChart className="w-5 h-5 text-gray-500" />
            </div>
            <div className="space-y-3">
              {analyticsData.traceabilityStats.map((stat, index) => (
                <div key={index} className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div
                      className={`w-3 h-3 rounded-full bg-${["blue", "green", "yellow", "purple", "gray"][index % 5]}-500`}
                    ></div>
                    <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                      {stat.type}
                    </span>
                  </div>
                  <div className="text-right">
                    <span className="text-sm font-semibold text-gray-900 dark:text-white">
                      {formatNumber(stat.count)}
                    </span>
                    <span className="text-xs text-gray-500 ml-2">
                      ({stat.percentage}%)
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </Card>
        </div>

        {/* Transaction Trends */}
        <Card className="mb-8">
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              Transaction Trends
            </h3>
            <LineChart className="w-5 h-5 text-gray-500" />
          </div>
          <div className="h-64 flex items-end justify-between gap-2">
            {analyticsData.transactionTrends.slice(-14).map((trend, index) => (
              <div key={index} className="flex flex-col items-center flex-1">
                <div
                  className="w-full bg-blue-500 rounded-t-sm min-h-[4px] transition-all duration-300 hover:bg-blue-600"
                  style={{
                    height: `${(trend.count / Math.max(...analyticsData.transactionTrends.map((t) => t.count))) * 200}px`,
                  }}
                  title={`${trend.count} transactions on ${trend.date}`}
                ></div>
                <span className="text-xs text-gray-500 mt-2 transform -rotate-45 origin-left">
                  {new Date(trend.date).toLocaleDateString("en-US", {
                    month: "short",
                    day: "numeric",
                  })}
                </span>
              </div>
            ))}
          </div>
        </Card>

        {/* Participant Activity */}
        <Card>
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              Top Participant Activity
            </h3>
            <Users className="w-5 h-5 text-gray-500" />
          </div>
          <div className="space-y-4">
            {analyticsData.participantActivity.map((participant, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg"
              >
                <div className="flex items-center gap-3">
                  <div className="w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center text-white font-semibold text-sm">
                    {participant.participant.charAt(0)}
                  </div>
                  <div>
                    <p className="font-medium text-gray-900 dark:text-white">
                      {participant.participant}
                    </p>
                    <p className="text-sm text-gray-500">
                      Last active: {participant.lastActive}
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="text-lg font-semibold text-gray-900 dark:text-white">
                    {participant.transactions}
                  </p>
                  <p className="text-sm text-gray-500">transactions</p>
                </div>
              </div>
            ))}
          </div>
        </Card>
      </div>
    </div>
  );
};

export default AnalyticsDashboard;
