// Statement Components for Wikidata Entity Viewer
// This module contains React components for rendering Wikidata statements/claims

// Check if React is available
if (typeof React === 'undefined') {
  console.error('React is not available. Make sure React is loaded before this script.');
}

/**
 * Reference Component
 * Renders a single reference that confirms or refutes a statement
 */
function Reference({ reference, getLabel, onEntityClick, onPropertyClick, selectedLanguage }) {
  const [isExpanded, setIsExpanded] = React.useState(false);
  
  const renderSnakValue = (snak) => {
    if (!snak.datavalue) return 'No value';
    
    const value = snak.datavalue.value;
    
    if (snak.datatype === 'wikibase-item' && value.id) {
      return (
        <a
          href={`entities.html#${value.id}`}
          className="statement-link"
          onClick={(e) => {
            e.preventDefault();
            if (onEntityClick) {
              onEntityClick(value.id);
            } else {
              window.location.href = `entities.html#${value.id}`;
            }
          }}
        >
          {getLabel(value.id)}
        </a>
      );
    } else if (snak.datatype === 'time') {
      return value.time;
    } else if (snak.datatype === 'string' || snak.datatype === 'external-id') {
      return value;
    } else if (snak.datatype === 'url') {
      return (
        <a href={value} target="_blank" rel="noopener noreferrer" className="statement-link">
          {value}
        </a>
      );
    }
    
    return String(value);
  };
  
  const mainSnaks = Object.entries(reference.snaks).slice(0, 2); // Show first 2 snaks by default
  const hasMore = Object.keys(reference.snaks).length > 2;
  
  return (
    <div style={{
      marginLeft: '20px',
      marginBottom: '8px',
      padding: '8px',
      border: '1px solid rgba(0, 255, 0, 0.3)',
      borderRadius: '4px',
      backgroundColor: 'rgba(0, 255, 0, 0.02)',
      fontSize: '0.9em'
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        <span style={{ color: 'var(--neon)', fontSize: '0.8em' }}>📍</span>
        <div style={{ flex: 1 }}>
          {mainSnaks.map(([propId, snakArray]) => (
            <div key={propId} style={{ marginBottom: '4px' }}>
              <a
                href={`properties.html#${propId}`}
                className="statement-link"
                onClick={(e) => {
                  if (onPropertyClick) {
                    e.preventDefault();
                    onPropertyClick(propId);
                  }
                }}
                style={{ fontWeight: 'bold', fontSize: '0.8em' }}
              >
                {getLabel(propId)}
              </a>
              {': '}
              {renderSnakValue(snakArray[0])}
            </div>
          ))}
          {hasMore && (
            <button
              onClick={() => setIsExpanded(!isExpanded)}
              style={{
                background: 'none',
                border: '1px solid var(--neon)',
                color: 'var(--neon)',
                padding: '2px 6px',
                fontSize: '0.7em',
                cursor: 'pointer',
                borderRadius: '2px'
              }}
            >
              {isExpanded ? 'Show less' : `+${Object.keys(reference.snaks).length - 2} more`}
            </button>
          )}
        </div>
      </div>
      
      {isExpanded && (
        <div style={{ marginTop: '8px', paddingTop: '8px', borderTop: '1px solid rgba(0, 255, 0, 0.2)' }}>
          {Object.entries(reference.snaks).slice(2).map(([propId, snakArray]) => (
            <div key={propId} style={{ marginBottom: '4px' }}>
              <a
                href={`properties.html#${propId}`}
                className="statement-link"
                onClick={(e) => {
                  if (onPropertyClick) {
                    e.preventDefault();
                    onPropertyClick(propId);
                  }
                }}
                style={{ fontWeight: 'bold', fontSize: '0.8em' }}
              >
                {getLabel(propId)}
              </a>
              {': '}
              {renderSnakValue(snakArray[0])}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

/**
 * Statement Component
 * Renders individual Wikidata statements with proper formatting and links
 * Now includes confirmations and refutations
 */
function Statement({
  items,
  claim,
  getLabel,
  onEntityClick,
  onPropertyClick,
  selectedLanguage
}) {
  const quotes = window.getQuotesForLanguage ? window.getQuotesForLanguage(selectedLanguage) : { open: '"', close: '"' };
  const renderItem = (item, index) => {
    if (item.startsWith('Q')) {
      return (
        <a
          key={`item-${index}`}
          href={`entities.html#${item}`}
          className="statement-link"
          onClick={(e) => {
            e.preventDefault();
            if (onEntityClick) {
              onEntityClick(item);
            } else {
              window.location.href = `entities.html#${item}`;
            }
          }}
        >
          {getLabel(item)}
        </a>
      );
    } else if (item.startsWith('P')) {
      return (
        <a
          key={`item-${index}`}
          href={`properties.html#${item}`}
          className="statement-link"
          onClick={(e) => {
            if (onPropertyClick) {
              e.preventDefault();
              onPropertyClick(item);
            }
          }}
        >
          {getLabel(item)}
        </a>
      );
    } else {
      return (
        <React.Fragment key={`item-${index}`}>
          {quotes.open}{item}{quotes.close}
        </React.Fragment>
      );
    }
  };
  const [showReferences, setShowReferences] = React.useState(false);
  const hasReferences = claim && claim.references && claim.references.length > 0;
  const isDeprecated = claim && claim.rank === 'deprecated';
  
  const getRankColor = (rank) => {
    switch (rank) {
      case 'preferred': return 'rgba(0, 255, 255, 0.1)'; // Cyan for preferred
      case 'deprecated': return 'rgba(255, 0, 0, 0.1)'; // Red for deprecated  
      default: return 'rgba(0, 255, 0, 0.05)'; // Green for normal
    }
  };
  
  const getRankBorder = (rank) => {
    switch (rank) {
      case 'preferred': return '1px solid var(--neon-selected)';
      case 'deprecated': return '1px solid #FF6B6B';
      default: return '1px solid var(--neon)';
    }
  };
  
  const getRankIcon = (rank) => {
    switch (rank) {
      case 'preferred': return '⭐';
      case 'deprecated': return '❌';
      default: return '';
    }
  };

  return (
    <div
      style={{
        marginBottom: '15px',
        padding: '12px',
        border: getRankBorder(claim?.rank),
        borderRadius: '6px',
        backgroundColor: getRankColor(claim?.rank)
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: hasReferences ? '8px' : '0' }}>
        {claim?.rank && claim.rank !== 'normal' && (
          <span title={`Rank: ${claim.rank}`} style={{ fontSize: '0.9em' }}>
            {getRankIcon(claim.rank)}
          </span>
        )}
        <div style={{ flex: 1 }}>
          {items.map((item, index) => (
            <React.Fragment key={`wrapper-${index}`}>
              {renderItem(item, index)}
              {index < items.length - 1 && ' '}
            </React.Fragment>
          ))}
        </div>
        {hasReferences && (
          <button
            onClick={() => setShowReferences(!showReferences)}
            style={{
              background: 'none',
              border: '1px solid var(--neon)',
              color: 'var(--neon)',
              padding: '4px 8px',
              fontSize: '0.8em',
              cursor: 'pointer',
              borderRadius: '4px',
              display: 'flex',
              alignItems: 'center',
              gap: '4px'
            }}
            title={`${claim.references.length} references available`}
          >
            <span>📚</span>
            <span>{claim.references.length}</span>
            <span>{showReferences ? '▼' : '▶'}</span>
          </button>
        )}
      </div>
      
      {showReferences && hasReferences && (
        <div style={{ marginTop: '8px' }}>
          <div style={{ 
            fontSize: '0.85em', 
            color: 'var(--neon)', 
            marginBottom: '8px',
            fontWeight: 'bold'
          }}>
            {isDeprecated ? 'Refutations:' : 'Confirmations:'}
          </div>
          {claim.references.map((reference, index) => (
            <Reference
              key={reference.hash || index}
              reference={reference}
              getLabel={getLabel}
              onEntityClick={onEntityClick}
              onPropertyClick={onPropertyClick}
              selectedLanguage={selectedLanguage}
            />
          ))}
        </div>
      )}
    </div>
  );
}

/**
 * Statements List Component
 * Renders a list of all statements for an entity
 */
function StatementsList({
  statements,
  subjectId,
  getLabel,
  onEntityClick,
  onPropertyClick,
  selectedLanguage
}) {
  const statementElements = [];

  console.log('Rendering statements with subjectId:', subjectId);

  for (const [propertyId, claims] of Object.entries(statements)) {
    if (Array.isArray(claims)) {
      claims.forEach((claim, index) => {
        // Create array of items for this statement
        const items = [];

        // Add entity (Q) or property (P)
        items.push(subjectId);

        // Add property (P)
        items.push(propertyId);

        // Add value (V) - could be entity ID or plain text
        if (claim.mainsnak.datavalue) {
          const value = claim.mainsnak.datavalue.value;
          if (claim.mainsnak.datatype === 'wikibase-item' && value.id) {
            items.push(value.id); // Entity ID
          } else if (claim.mainsnak.datatype === 'time') {
            items.push(value.time);
          } else if (claim.mainsnak.datatype === 'quantity') {
            items.push(value.amount);
          } else if (claim.mainsnak.datatype === 'string') {
            items.push(value);
          } else if (claim.mainsnak.datatype === 'url') {
            items.push(value);
          } else if (claim.mainsnak.datatype === 'external-id') {
            items.push(value);
          } else if (claim.mainsnak.datatype === 'commonsMedia') {
            items.push(value);
          } else if (claim.mainsnak.datatype === 'geo-coordinate') {
            items.push(`${value.latitude}, ${value.longitude}`);
          } else if (claim.mainsnak.datatype === 'monolingualtext') {
            items.push(value.text);
          } else if (claim.mainsnak.datatype === 'wikibase-lexeme') {
            items.push(value.id);
          } else if (claim.mainsnak.datatype === 'wikibase-form') {
            items.push(value.id);
          } else if (claim.mainsnak.datatype === 'wikibase-sense') {
            items.push(value.id);
          } else {
            items.push('Unknown value type');
          }
        } else {
          items.push('No value');
        }

        statementElements.push(
          <Statement
            key={`${propertyId}-${index}`}
            items={items}
            claim={claim}
            getLabel={getLabel}
            onEntityClick={onEntityClick}
            onPropertyClick={onPropertyClick}
            selectedLanguage={selectedLanguage}
          />
        );
      });
    }
  }

  return statementElements;
}

/**
 * Statements Section Component
 * Renders the complete statements section with title and conditional rendering
 */
function StatementsSection({
  statements,
  subjectId,
  getLabel,
  onEntityClick,
  onPropertyClick,
  selectedLanguage
}) {
  const hasStatements = Object.keys(statements).length > 0;

  if (!hasStatements) {
    return (
      <div>
        <h2>Statements</h2>
        <p>No statements available</p>
      </div>
    );
  }

  return (
    <div>
      <h2>Statements</h2>
      <div style={{ marginTop: '20px' }}>
        <StatementsList
          statements={statements}
          subjectId={subjectId}
          getLabel={getLabel}
          onEntityClick={onEntityClick}
          onPropertyClick={onPropertyClick}
          selectedLanguage={selectedLanguage}
        />
      </div>
    </div>
  );
}

// Export components to global scope
window.StatementComponents = {
  Reference: Reference,
  Statement: Statement,
  StatementsList: StatementsList,
  StatementsSection: StatementsSection
}; 