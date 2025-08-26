import React from 'react';
import './TabNavigation.css';
import { TabType } from '../App';

interface TabNavigationProps {
  activeTab: TabType;
  onTabChange: (tab: TabType) => void;
}

interface Tab {
  id: TabType;
  label: string;
  icon: string;
  description: string;
}

const tabs: Tab[] = [
  {
    id: 'ontology',
    label: 'Ontology',
    icon: '🏗️',
    description: 'Define classes and properties'
  },
  {
    id: 'rdf',
    label: 'RDF Store',
    icon: '🗄️',
    description: 'Manage RDF triples'
  },
  {
    id: 'knowledge-graph',
    label: 'Knowledge Graph',
    icon: '🕸️',
    description: 'Visualize relationships'
  },
  {
    id: 'provenance',
    label: 'Provenance',
    icon: '📜',
    description: 'Track PROV-O lineage'
  },
  {
    id: 'queries',
    label: 'Queries',
    icon: '🔍',
    description: 'Traceability analysis'
  },
  {
    id: 'data',
    label: 'Data Manager',
    icon: '📊',
    description: 'Import/Export data'
  }
];

export const TabNavigation: React.FC<TabNavigationProps> = ({ activeTab, onTabChange }) => {
  return (
    <nav className="tab-navigation">
      <div className="tab-container">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            className={`tab ${activeTab === tab.id ? 'tab-active' : ''}`}
            onClick={() => onTabChange(tab.id)}
          >
            <span className="tab-icon">{tab.icon}</span>
            <div className="tab-content">
              <span className="tab-label">{tab.label}</span>
              <span className="tab-description">{tab.description}</span>
            </div>
          </button>
        ))}
      </div>
    </nav>
  );
};