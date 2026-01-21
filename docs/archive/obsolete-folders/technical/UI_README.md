# ProvChain Web UI

A modern, responsive web interface for the ProvChain blockchain provenance system.

## Features

### 🎯 Dashboard
- Real-time blockchain statistics
- System health monitoring
- Recent transactions overview
- Network status indicators

### 🔍 Block Explorer
- Browse all blockchain blocks
- Search blocks by index or hash
- View block details and metadata
- Real-time block updates

### 📦 Product Traceability
- Trace products through the supply chain
- View product journey timeline
- Environmental data tracking
- Certification management

### 🔎 SPARQL Query Interface
- Execute SPARQL queries on RDF data
- Pre-built query templates
- Real-time query execution
- Results visualization

### 💼 Transaction Management
- Add new RDF triples to blockchain
- View transaction history
- Real-time transaction updates

### 🔐 Authentication
- Secure login system
- JWT token-based authentication
- Role-based access control

## Getting Started

### Prerequisites
- Rust 1.70+ installed
- Modern web browser (Chrome, Firefox, Safari, Edge)

### Quick Start

1. **Run the demo with sample data:**
   ```bash
   cargo run --bin demo_ui
   ```

2. **Open your browser and navigate to:**
   ```
   http://localhost:8080
   ```

3. **Explore the features:**
   - Dashboard: View system overview
   - Blocks: Browse blockchain blocks
   - Traceability: Try batch IDs "BATCH001" or "BATCH002"
   - SPARQL: Use query templates or write custom queries
   - Transactions: Add new data to the blockchain

### Manual Setup

1. **Start the web server:**
   ```bash
   cargo run
   ```

2. **The server will start on port 8080 by default**

3. **Access the web interface at:**
   ```
   http://localhost:8080
   ```

## UI Components

### Navigation
- **Dashboard**: System overview and statistics
- **Blocks**: Blockchain explorer
- **Traceability**: Product tracking interface
- **SPARQL**: Query interface for RDF data
- **Transactions**: Add and view transactions

### Authentication
- Login with any username/password in demo mode
- JWT tokens for secure API access
- Automatic session management

### Responsive Design
- Mobile-friendly interface
- Tablet and desktop optimized
- Modern glassmorphism design
- Dark/light theme support

## API Integration

The UI integrates with the following API endpoints:

### Public Endpoints
- `GET /health` - Health check
- `POST /auth/login` - Authentication

### Protected Endpoints (require authentication)
- `GET /api/blockchain/status` - Blockchain status
- `GET /api/blockchain/blocks` - All blocks
- `GET /api/blockchain/blocks/:index` - Specific block
- `GET /api/blockchain/validate` - Validate blockchain
- `GET /api/transactions/recent` - Recent transactions
- `POST /api/sparql/query` - Execute SPARQL query
- `GET /api/products/trace` - Product traceability
- `POST /api/blockchain/add-triple` - Add new triple

## Sample Data

The demo includes sample supply chain data:

### Products
- **BATCH001**: Organic Coffee Beans from Colombia
- **BATCH002**: Fair Trade Cocoa Beans from Ecuador

### Features Demonstrated
- Product origins and current locations
- Environmental data (temperature, humidity, CO2 footprint)
- Supply chain events and actors
- Certifications (Organic, Fair Trade, Rainforest Alliance)
- Blockchain metadata

### Sample SPARQL Queries

**Get all products:**
```sparql
PREFIX tc: <http://example.org/tracechain#>
SELECT ?batch ?product ?origin
WHERE {
    ?batch tc:product ?product .
    ?batch tc:origin ?origin .
}
```

**Trace a specific batch:**
```sparql
PREFIX tc: <http://example.org/tracechain#>
SELECT ?batch ?product ?origin ?location ?status
WHERE {
    ?batch tc:batchId "BATCH001" .
    ?batch tc:product ?product .
    OPTIONAL { ?batch tc:origin ?origin }
    OPTIONAL { ?batch tc:currentLocation ?location }
    OPTIONAL { ?batch tc:status ?status }
}
```

**Environmental conditions:**
```sparql
PREFIX tc: <http://example.org/tracechain#>
SELECT ?batch ?temperature ?humidity ?co2
WHERE {
    ?batch tc:environmentalData ?envData .
    OPTIONAL { ?envData tc:temperature ?temperature }
    OPTIONAL { ?envData tc:humidity ?humidity }
    OPTIONAL { ?envData tc:co2Footprint ?co2 }
}
```

## Technology Stack

### Frontend
- **HTML5**: Semantic markup
- **CSS3**: Modern styling with glassmorphism effects
- **JavaScript (ES6+)**: Interactive functionality
- **Font Awesome**: Icons
- **Google Fonts**: Typography

### Backend
- **Rust**: Core blockchain implementation
- **Axum**: Web framework
- **Tower HTTP**: Static file serving and CORS
- **Oxigraph**: RDF/SPARQL engine
- **JWT**: Authentication tokens

### Features
- **Responsive Design**: Works on all devices
- **Real-time Updates**: Live data refresh
- **Toast Notifications**: User feedback
- **Loading States**: Better UX
- **Error Handling**: Graceful error management

## Customization

### Styling
- Modify `static/styles.css` for custom themes
- Update CSS variables for color schemes
- Responsive breakpoints can be adjusted

### Functionality
- Extend `static/app.js` for new features
- Add new API endpoints in Rust backend
- Customize SPARQL query templates

### Configuration
- Server port can be changed in the code
- API base URL is configurable
- Authentication settings are customizable

## Development

### File Structure
```
static/
├── index.html      # Main HTML file
├── styles.css      # CSS styles
└── app.js         # JavaScript functionality

src/web/
├── server.rs      # Web server implementation
├── handlers.rs    # API handlers
├── models.rs      # Data models
└── auth.rs        # Authentication
```

### Adding New Features

1. **Frontend**: Add HTML elements and JavaScript handlers
2. **Backend**: Create new API endpoints in handlers.rs
3. **Models**: Define data structures in models.rs
4. **Integration**: Connect frontend to backend APIs

## Troubleshooting

### Common Issues

**Server won't start:**
- Check if port 8080 is available
- Ensure all dependencies are installed
- Check Rust version compatibility

**UI not loading:**
- Verify static files are in the correct directory
- Check browser console for errors
- Ensure server is running

**API calls failing:**
- Check authentication status
- Verify API endpoints are correct
- Check CORS configuration

**SPARQL queries not working:**
- Verify query syntax
- Check RDF data is loaded
- Ensure proper prefixes are used

### Browser Compatibility
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
