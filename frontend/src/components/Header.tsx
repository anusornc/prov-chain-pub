import React from "react";
import "./Header.css";
import Button from "./ui/Button";
import { useTheme } from "../hooks/useTheme";

type TabType =
  | "ontology"
  | "rdf"
  | "knowledge-graph"
  | "provenance"
  | "queries"
  | "data";

interface HeaderProps {
  activeTab: TabType;
  onTabChange: (tab: TabType) => void;
}

const Header: React.FC<HeaderProps> = ({ activeTab, onTabChange }) => {
  const { theme, toggleTheme } = useTheme();

  const tabs = [
    { id: "ontology", label: "Ontology", icon: "🏛️" },
    { id: "rdf", label: "RDF", icon: "🔗" },
    { id: "knowledge-graph", label: "Knowledge Graph", icon: "🧠" },
    { id: "provenance", label: "Provenance", icon: "📜" },
    { id: "queries", label: "Queries", icon: "🔍" },
    { id: "data", label: "Data", icon: "📊" },
  ];

  return (
    <header className="header">
      <div className="header-content">
        <div className="logo">
          <h1>ProvChainOrg</h1>
          <span className="subtitle">Semantic Blockchain Explorer</span>
        </div>

        <nav className="tabs">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              className={`tab ${activeTab === tab.id ? "active" : ""}`}
              onClick={() => onTabChange(tab.id as TabType)}
            >
              <span className="tab-icon">{tab.icon}</span>
              <span className="tab-label">{tab.label}</span>
            </button>
          ))}
        </nav>

        <div className="header-actions">
          <div className="status-indicator">
            <span className="status-dot active"></span>
            <span>Connected</span>
          </div>
          <Button
            variant="secondary"
            size="sm"
            onClick={toggleTheme}
            aria-label={`Switch to ${theme === "light" ? "dark" : "light"} mode`}
          >
            {theme === "light" ? "🌙" : "☀️"}
          </Button>
          <Button variant="secondary" size="sm">
            ⚙️ Settings
          </Button>
        </div>
      </div>
    </header>
  );
};

export default Header;
