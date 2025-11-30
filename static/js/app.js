
let cy;

document.addEventListener('DOMContentLoaded', () => {
    initCy();
    // Load initial graph data if available, or just wait for user interaction
    // loadGraph(); 
});

function initCy() {
    cy = cytoscape({
        container: document.getElementById('cy'),
        style: [
            {
                selector: 'node',
                style: {
                    'label': 'data(label)',
                    'text-valign': 'center',
                    'color': '#fff',
                    'text-outline-width': 2,
                    'text-outline-color': '#555',
                    'background-color': '#95a5a6',
                    'width': 40,
                    'height': 40
                }
            },
            {
                selector: 'node[type="ProductBatch"]',
                style: { 'background-color': '#3498db' }
            },
            {
                selector: 'node[type="ProcessingActivity"]',
                style: { 'background-color': '#e74c3c', 'shape': 'rectangle' }
            },
            {
                selector: 'node[type="TransportActivity"]',
                style: { 'background-color': '#e67e22', 'shape': 'rectangle' }
            },
            {
                selector: 'node[type="Farmer"]',
                style: { 'background-color': '#2ecc71', 'shape': 'triangle' }
            },
            {
                selector: 'node[type="Manufacturer"]',
                style: { 'background-color': '#27ae60', 'shape': 'triangle' }
            },
            {
                selector: 'node[type="Retailer"]',
                style: { 'background-color': '#16a085', 'shape': 'triangle' }
            },
            {
                selector: 'edge',
                style: {
                    'width': 2,
                    'line-color': '#ccc',
                    'target-arrow-color': '#ccc',
                    'target-arrow-shape': 'triangle',
                    'curve-style': 'bezier',
                    'label': 'data(label)',
                    'font-size': 10,
                    'text-rotation': 'autorotate',
                    'text-background-opacity': 1,
                    'text-background-color': '#fff'
                }
            },
            {
                selector: '.highlighted',
                style: {
                    'background-color': '#f1c40f',
                    'line-color': '#f1c40f',
                    'target-arrow-color': '#f1c40f',
                    'transition-property': 'background-color, line-color, target-arrow-color',
                    'transition-duration': '0.5s'
                }
            }
        ],
        layout: {
            name: 'cose',
            animate: true
        }
    });

    cy.on('tap', 'node', function (evt) {
        const node = evt.target;
        document.getElementById('node-data').textContent = JSON.stringify(node.data(), null, 2);
    });
}

async function loadGraph() {
    const status = document.getElementById('status');
    status.style.display = 'block';
    status.textContent = 'Loading graph...';
    status.className = 'status';

    const rootItem = document.getElementById('root-item').value;
    let url = '/api/knowledge-graph';
    if (rootItem) {
        url += `?item_id=${encodeURIComponent(rootItem)}`;
    } else {
        // Fallback to getting all products if no root item specified, 
        // as getting the ENTIRE graph might be too heavy without pagination
        // For this demo, we'll try to fetch a default set if empty
        // Or we can implement a 'get all' endpoint. 
        // Let's assume /api/knowledge-graph handles empty params gracefully or we provide a default
        url += `?item_id=http://provchain.org/trace-test%23Batch3`; // Default for demo
    }

    try {
        const response = await fetch(url);
        if (!response.ok) throw new Error('Failed to fetch graph data');
        const data = await response.json();

        cy.elements().remove();

        // Transform data for Cytoscape
        const elements = [];
        data.nodes.forEach(n => {
            elements.push({
                group: 'nodes',
                data: {
                    id: n.id,
                    label: n.label,
                    type: n.properties.type || 'unknown', // Assuming type is in properties
                    ...n.properties
                }
            });
        });

        data.edges.forEach(e => {
            elements.push({
                group: 'edges',
                data: {
                    source: e.source,
                    target: e.target,
                    label: e.label
                }
            });
        });

        cy.add(elements);
        cy.layout({ name: 'cose', animate: true }).run();

        status.textContent = `Graph loaded: ${data.nodes.length} nodes, ${data.edges.length} edges`;
        status.className = 'status success';
    } catch (error) {
        console.error(error);
        status.textContent = `Error: ${error.message}`;
        status.className = 'status error';
    }
}

async function tracePath() {
    const from = document.getElementById('from').value;
    const to = document.getElementById('to').value;
    const status = document.getElementById('status');

    if (!from || !to) {
        alert('Please enter both From and To URIs');
        return;
    }

    status.style.display = 'block';
    status.textContent = 'Tracing path...';
    status.className = 'status';

    try {
        const response = await fetch(`/api/trace?from=${encodeURIComponent(from)}&to=${encodeURIComponent(to)}`);
        if (!response.ok) throw new Error('Trace failed');
        const data = await response.json();

        if (data.found) {
            status.textContent = `Path found! Length: ${data.length}`;
            status.className = 'status success';

            // Highlight path
            cy.elements().removeClass('highlighted');

            // Add any missing nodes from the path to the graph
            // (In a real app, we might want to fetch the subgraph for the path)

            // Highlight nodes in path
            data.path.forEach((nodeUri, index) => {
                const node = cy.getElementById(nodeUri);
                if (node.length > 0) {
                    node.addClass('highlighted');

                    // Highlight edge to next node
                    if (index < data.path.length - 1) {
                        const nextUri = data.path[index + 1];
                        cy.edges(`[source="${nodeUri}"][target="${nextUri}"]`).addClass('highlighted');
                        // Also check reverse direction just in case
                        cy.edges(`[source="${nextUri}"][target="${nodeUri}"]`).addClass('highlighted');
                    }
                }
            });

        } else {
            status.textContent = 'No path found.';
            status.className = 'status error';
        }
    } catch (error) {
        console.error(error);
        status.textContent = `Error: ${error.message}`;
        status.className = 'status error';
    }
}
