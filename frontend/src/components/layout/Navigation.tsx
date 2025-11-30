import React, { useState } from "react";
import {
  Box,
  Search,
  Activity,
  GitBranch,
  Database,
  Settings,
  Menu,
  X,
  Home,
  Users,
  BarChart3,
  Shield,
} from "lucide-react";
import type { NavigationItem } from "../../types";

interface NavigationProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
  className?: string;
}

const navigationItems: NavigationItem[] = [
  {
    id: "dashboard",
    label: "Dashboard",
    icon: "Home",
    path: "/dashboard",
  },
  {
    id: "explorer",
    label: "Block Explorer",
    icon: "Box",
    path: "/explorer",
    children: [
      { id: "blocks", label: "Blocks", icon: "Box", path: "/explorer/blocks" },
      {
        id: "transactions",
        label: "Transactions",
        icon: "Activity",
        path: "/explorer/transactions",
      },
      {
        id: "search",
        label: "Search",
        icon: "Search",
        path: "/explorer/search",
      },
    ],
  },
  {
    id: "traceability",
    label: "Traceability",
    icon: "GitBranch",
    path: "/traceability",
    children: [
      {
        id: "items",
        label: "Items",
        icon: "Database",
        path: "/traceability/items",
      },
      {
        id: "knowledge-graph",
        label: "Knowledge Graph",
        icon: "GitBranch",
        path: "/traceability/graph",
      },
      {
        id: "timeline",
        label: "Timeline",
        icon: "Activity",
        path: "/traceability/timeline",
      },
    ],
  },
  {
    id: "participants",
    label: "Participants",
    icon: "Users",
    path: "/participants",
  },
  {
    id: "analytics",
    label: "Analytics",
    icon: "BarChart3",
    path: "/analytics",
  },
  {
    id: "semantic",
    label: "Semantic Queries",
    icon: "Database",
    path: "/semantic",
    children: [
      {
        id: "sparql",
        label: "SPARQL Editor",
        icon: "Database",
        path: "/semantic/sparql",
      },
      {
        id: "query-builder",
        label: "Query Builder",
        icon: "Settings",
        path: "/semantic/builder",
      },
      {
        id: "ontology",
        label: "Ontology",
        icon: "Shield",
        path: "/semantic/ontology",
      },
    ],
  },
];

const iconMap = {
  Home,
  Box,
  Search,
  Activity,
  GitBranch,
  Database,
  Settings,
  Users,
  BarChart3,
  Shield,
};

const Navigation: React.FC<NavigationProps> = ({
  activeTab,
  onTabChange,
  className = "",
}) => {
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);
  const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set());

  const toggleExpanded = (itemId: string) => {
    const newExpanded = new Set(expandedItems);
    if (newExpanded.has(itemId)) {
      newExpanded.delete(itemId);
    } else {
      newExpanded.add(itemId);
    }
    setExpandedItems(newExpanded);
  };

  const handleItemClick = (item: NavigationItem) => {
    if (item.children && item.children.length > 0) {
      toggleExpanded(item.id);
    } else {
      onTabChange(item.id);
      setIsMobileMenuOpen(false);
    }
  };

  const renderNavigationItem = (item: NavigationItem, level = 0) => {
    const IconComponent =
      iconMap[item.icon as keyof typeof iconMap] || Database;
    const isActive = activeTab === item.id;
    const isExpanded = expandedItems.has(item.id);
    const hasChildren = item.children && item.children.length > 0;

    return (
      <div key={item.id} className="mb-1">
        <button
          onClick={() => handleItemClick(item)}
          className={`
            w-full flex items-center justify-between px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-200
            ${level > 0 ? "ml-4 pl-6" : ""}
            ${
              isActive
                ? "bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300 shadow-sm"
                : "text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 hover:text-gray-900 dark:hover:text-white"
            }
          `}
        >
          <div className="flex items-center">
            <IconComponent className="w-5 h-5 mr-3 flex-shrink-0" />
            <span className="truncate">{item.label}</span>
            {item.badge && (
              <span className="ml-2 px-2 py-0.5 text-xs bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300 rounded-full">
                {item.badge}
              </span>
            )}
          </div>
          {hasChildren && (
            <div
              className={`transform transition-transform duration-200 ${isExpanded ? "rotate-90" : ""}`}
            >
              <svg
                className="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M9 5l7 7-7 7"
                />
              </svg>
            </div>
          )}
        </button>

        {hasChildren && isExpanded && (
          <div className="mt-1 space-y-1">
            {item.children!.map((child) =>
              renderNavigationItem(child, level + 1),
            )}
          </div>
        )}
      </div>
    );
  };

  return (
    <>
      {/* Mobile menu button */}
      <div className="lg:hidden fixed top-4 left-4 z-50">
        <button
          onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
          className="p-2 rounded-lg bg-white dark:bg-gray-800 shadow-lg border border-gray-200 dark:border-gray-700"
        >
          {isMobileMenuOpen ? (
            <X className="w-6 h-6 text-gray-600 dark:text-gray-300" />
          ) : (
            <Menu className="w-6 h-6 text-gray-600 dark:text-gray-300" />
          )}
        </button>
      </div>

      {/* Mobile overlay */}
      {isMobileMenuOpen && (
        <div
          className="lg:hidden fixed inset-0 bg-black bg-opacity-50 z-40"
          onClick={() => setIsMobileMenuOpen(false)}
        />
      )}

      {/* Navigation sidebar */}
      <nav
        className={`
        fixed lg:static inset-y-0 left-0 z-40 w-64 bg-white dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700 transform transition-transform duration-300 ease-in-out
        ${isMobileMenuOpen ? "translate-x-0" : "-translate-x-full lg:translate-x-0"}
        ${className}
      `}
      >
        <div className="flex flex-col h-full">
          {/* Header */}
          <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
            <div className="flex items-center">
              <div className="w-8 h-8 bg-gradient-to-br from-primary-500 to-primary-600 rounded-lg flex items-center justify-center">
                <Box className="w-5 h-5 text-white" />
              </div>
              <div className="ml-3">
                <h1 className="text-lg font-bold text-gray-900 dark:text-white">
                  ProvChain
                </h1>
                <p className="text-xs text-gray-500 dark:text-gray-400">
                  Explorer
                </p>
              </div>
            </div>
          </div>

          {/* Navigation items */}
          <div className="flex-1 overflow-y-auto p-4">
            <div className="space-y-2">
              {navigationItems.map((item) => renderNavigationItem(item))}
            </div>
          </div>

          {/* Footer */}
          <div className="p-4 border-t border-gray-200 dark:border-gray-700">
            <div className="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
              <span>v1.0.0</span>
              <div className="flex items-center">
                <div className="w-2 h-2 bg-green-400 rounded-full mr-2"></div>
                <span>Online</span>
              </div>
            </div>
          </div>
        </div>
      </nav>
    </>
  );
};

export default Navigation;
