#!/usr/bin/env node

// Links API Test Suite
// Tests the new Wikidata Links API functionality
// Run with: bun links-api-test.mjs

import { linksApi, WikidataLinksAPI, client } from './wikidata-api.js';

/**
 * Links API Test Suite
 * Comprehensive tests for the new Wikidata Links API functionality
 */
class LinksAPITestSuite {
  constructor() {
    this.linksApi = linksApi;
    this.testResults = [];
    this.totalTests = 0;
    this.passedTests = 0;
  }

  /**
   * Log test results
   */
  logTest(testName, passed, message = '') {
    this.totalTests++;
    if (passed) {
      this.passedTests++;
      console.log(`✅ ${testName}`);
    } else {
      console.log(`❌ ${testName}: ${message}`);
    }
    this.testResults.push({ testName, passed, message });
  }

  /**
   * Test basic outgoing links functionality
   */
  async testOutgoingLinks() {
    console.log('\n🔗 Testing Outgoing Links...');
    
    try {
      // Test with Douglas Adams (Q42) - well-known entity with many links
      const result = await this.linksApi.getOutgoingLinks('Q42', 'en', { limit: 5 });
      
      this.logTest('Outgoing links returns result object', result && typeof result === 'object');
      this.logTest('Result has entity field', result.entity === 'Q42');
      this.logTest('Result has links array', Array.isArray(result.links));
      this.logTest('Result has total count', typeof result.total === 'number');
      this.logTest('Links are limited to 5', result.links.length <= 5);
      
      if (result.links.length > 0) {
        const firstLink = result.links[0];
        this.logTest('Link has required fields', 
          firstLink.subject && firstLink.property && firstLink.hasOwnProperty('value'));
        this.logTest('Link has value type', firstLink.valueType !== undefined);
        
        console.log(`   Sample link: ${firstLink.subject} → ${firstLink.property} → ${firstLink.value} (${firstLink.valueType})`);
      }
      
    } catch (error) {
      this.logTest('Outgoing links basic test', false, error.message);
    }
  }

  /**
   * Test outgoing links with property filter
   */
  async testOutgoingLinksWithFilter() {
    console.log('\n🔍 Testing Outgoing Links with Property Filter...');
    
    try {
      // Test filtering for specific properties (occupation, instance of)
      const result = await this.linksApi.getOutgoingLinks('Q42', 'en', { 
        propertyFilter: ['P31', 'P106'], // instance of, occupation
        includeLabels: true
      });
      
      this.logTest('Filtered links returns results', result && result.links);
      
      // Check that all returned links match the filter
      const allLinksMatchFilter = result.links.every(link => 
        ['P31', 'P106'].includes(link.property)
      );
      this.logTest('All links match property filter', allLinksMatchFilter);
      
      // Check for labels
      if (result.links.length > 0) {
        const hasLabels = result.links.some(link => link.propertyLabel);
        this.logTest('Links include property labels', hasLabels);
        
        console.log('   Filtered links:');
        result.links.forEach(link => {
          console.log(`     ${link.propertyLabel || link.property}: ${link.valueLabel || link.value}`);
        });
      }
      
    } catch (error) {
      this.logTest('Outgoing links with filter test', false, error.message);
    }
  }

  /**
   * Test linked entities functionality
   */
  async testLinkedEntities() {
    console.log('\n👥 Testing Linked Entities...');
    
    try {
      // Get entities linked to Douglas Adams via "occupation" property
      const result = await this.linksApi.getLinkedEntities('Q42', 'P106', 'en');
      
      this.logTest('Linked entities returns array', Array.isArray(result));
      
      if (result.length > 0) {
        const firstEntity = result[0];
        this.logTest('Linked entity has required fields',
          firstEntity.id && firstEntity.property === 'P106');
        
        console.log('   Linked entities (occupations):');
        result.forEach(entity => {
          console.log(`     ${entity.label || entity.id} (${entity.id})`);
        });
      }
      
    } catch (error) {
      this.logTest('Linked entities test', false, error.message);
    }
  }

  /**
   * Test relationships between entities
   */
  async testRelationshipsBetween() {
    console.log('\n🔗 Testing Relationships Between Entities...');
    
    try {
      // Test relationships between Douglas Adams (Q42) and The Hitchhiker's Guide (Q3107329)
      const result = await this.linksApi.getRelationshipsBetween('Q42', 'Q3107329', 'en');
      
      this.logTest('Relationships returns result object', result && typeof result === 'object');
      this.logTest('Result has entity1 and entity2 fields', 
        result.entity1 === 'Q42' && result.entity2 === 'Q3107329');
      this.logTest('Result has relationship arrays',
        Array.isArray(result.entity1ToEntity2) && Array.isArray(result.entity2ToEntity1));
      this.logTest('Result has total count', typeof result.totalRelationships === 'number');
      
      console.log(`   Total relationships found: ${result.totalRelationships}`);
      console.log(`   Bidirectional: ${result.bidirectional}`);
      
      if (result.entity1ToEntity2.length > 0) {
        console.log('   Q42 → Q3107329:');
        result.entity1ToEntity2.forEach(rel => {
          console.log(`     via ${rel.propertyLabel || rel.property}`);
        });
      }
      
    } catch (error) {
      this.logTest('Relationships between entities test', false, error.message);
    }
  }

  /**
   * Test knowledge graph functionality
   */
  async testKnowledgeGraph() {
    console.log('\n🕸️  Testing Knowledge Graph...');
    
    try {
      // Build a small knowledge graph around Douglas Adams
      const result = await this.linksApi.getKnowledgeGraph('Q42', 1, 'en', { 
        maxNodes: 10,
        propertyFilter: ['P31', 'P106', 'P800'] // instance of, occupation, notable work
      });
      
      this.logTest('Knowledge graph returns result object', result && typeof result === 'object');
      this.logTest('Result has start entity', result.startEntity === 'Q42');
      this.logTest('Result has nodes array', Array.isArray(result.nodes));
      this.logTest('Result has edges array', Array.isArray(result.edges));
      this.logTest('Result has counts', 
        typeof result.totalNodes === 'number' && typeof result.totalEdges === 'number');
      
      console.log(`   Knowledge graph: ${result.totalNodes} nodes, ${result.totalEdges} edges`);
      console.log(`   Max depth: ${result.maxDepth}`);
      
      if (result.nodes.length > 0) {
        console.log('   Sample nodes:');
        result.nodes.slice(0, 3).forEach(node => {
          console.log(`     ${node.label || node.id} (depth: ${node.depth})`);
        });
      }
      
      if (result.edges.length > 0) {
        console.log('   Sample edges:');
        result.edges.slice(0, 3).forEach(edge => {
          console.log(`     ${edge.source} → ${edge.propertyLabel || edge.property} → ${edge.target}`);
        });
      }
      
    } catch (error) {
      this.logTest('Knowledge graph test', false, error.message);
    }
  }

  /**
   * Test incoming links functionality (note: limited implementation)
   */
  async testIncomingLinks() {
    console.log('\n⬅️  Testing Incoming Links...');
    
    try {
      const result = await this.linksApi.getIncomingLinks('Q42', 'en');
      
      this.logTest('Incoming links returns result object', result && typeof result === 'object');
      this.logTest('Result has entity field', result.entity === 'Q42');
      this.logTest('Result includes implementation note', result.note);
      
      console.log(`   Note: ${result.note}`);
      
    } catch (error) {
      this.logTest('Incoming links test', false, error.message);
    }
  }

  /**
   * Test error handling
   */
  async testErrorHandling() {
    console.log('\n⚠️  Testing Error Handling...');
    
    try {
      // Test with non-existent entity
      const result = await this.linksApi.getOutgoingLinks('Q999999999', 'en');
      this.logTest('Non-existent entity handled gracefully', 
        result && result.links && result.links.length === 0);
      
    } catch (error) {
      this.logTest('Error handling for non-existent entity', false, error.message);
    }
    
    try {
      // Test with invalid parameters
      const result = await this.linksApi.getLinkedEntities('Q42', 'InvalidProperty', 'en');
      this.logTest('Invalid property handled gracefully', 
        Array.isArray(result) && result.length === 0);
      
    } catch (error) {
      this.logTest('Error handling for invalid property', false, error.message);
    }
  }

  /**
   * Test performance with larger datasets
   */
  async testPerformance() {
    console.log('\n⚡ Testing Performance...');
    
    try {
      const startTime = Date.now();
      
      // Test with a well-connected entity and reasonable limits
      const result = await this.linksApi.getOutgoingLinks('Q42', 'en', { 
        limit: 20,
        includeLabels: true
      });
      
      const endTime = Date.now();
      const duration = endTime - startTime;
      
      this.logTest('Performance test completed', result && result.links);
      this.logTest('Performance under 5 seconds', duration < 5000);
      
      console.log(`   Retrieved ${result.links.length} links in ${duration}ms`);
      
    } catch (error) {
      this.logTest('Performance test', false, error.message);
    }
  }

  /**
   * Run all tests
   */
  async runAllTests() {
    console.log('🧪 Starting Links API Test Suite...\n');
    
    await this.testOutgoingLinks();
    await this.testOutgoingLinksWithFilter();
    await this.testLinkedEntities();
    await this.testRelationshipsBetween();
    await this.testKnowledgeGraph();
    await this.testIncomingLinks();
    await this.testErrorHandling();
    await this.testPerformance();
    
    this.printSummary();
  }

  /**
   * Print test summary
   */
  printSummary() {
    console.log('\n📊 Test Summary');
    console.log('='.repeat(50));
    console.log(`Total tests: ${this.totalTests}`);
    console.log(`Passed: ${this.passedTests}`);
    console.log(`Failed: ${this.totalTests - this.passedTests}`);
    console.log(`Success rate: ${((this.passedTests / this.totalTests) * 100).toFixed(1)}%`);
    
    if (this.passedTests === this.totalTests) {
      console.log('\n🎉 All tests passed! Links API is working correctly.');
    } else {
      console.log('\n⚠️  Some tests failed. Check the output above for details.');
    }
    
    // Show failed tests
    const failedTests = this.testResults.filter(t => !t.passed);
    if (failedTests.length > 0) {
      console.log('\n❌ Failed tests:');
      failedTests.forEach(test => {
        console.log(`   - ${test.testName}: ${test.message}`);
      });
    }
  }
}

// Run the test suite
async function main() {
  const testSuite = new LinksAPITestSuite();
  await testSuite.runAllTests();
}

// Execute if run directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(error => {
    console.error('Test suite failed:', error);
    process.exit(1);
  });
}

export { LinksAPITestSuite };