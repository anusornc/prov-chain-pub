import React from 'react';
import './Header.css';

export const Header: React.FC = () => {
  return (
    <header className="header">
      <div className="header-content">
        <div className="header-left">
          <h1 className="header-title">
            <span className="header-icon">🔗</span>
            Semantic Traceability System
          </h1>
          <p className="header-subtitle">
            RDF • RDFS • OWL • PROV-O • Knowledge Graphs
          </p>
        </div>
        <div className="header-right">
          <div className="status-indicator active"></div>
          <span className="status-text">System Active</span>
        </div>
      </div>
    </header>
  );
};