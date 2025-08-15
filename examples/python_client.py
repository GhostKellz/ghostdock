#!/usr/bin/env python3
"""
GhostDock Registry Python Client Example

This example demonstrates how to interact with the GhostDock Registry
using Python and the requests library.
"""

import base64
import json
import hashlib
import requests
from typing import Dict, List, Optional


class GhostDockClient:
    """Python client for GhostDock Registry"""
    
    def __init__(self, base_url: str, username: str = None, password: str = None, token: str = None):
        """
        Initialize the client
        
        Args:
            base_url: Registry base URL (e.g., https://registry.example.com)
            username: Username for authentication
            password: Password for authentication  
            token: Personal access token (alternative to username/password)
        """
        self.base_url = base_url.rstrip('/')
        self.session = requests.Session()
        
        if token:
            self.session.headers['Authorization'] = f'Bearer {token}'
        elif username and password:
            self.authenticate(username, password)
    
    def authenticate(self, username: str, password: str) -> str:
        """Authenticate with username and password"""
        response = self.session.post(
            f"{self.base_url}/auth",
            json={"username": username, "password": password}
        )
        response.raise_for_status()
        
        token = response.json()["token"]
        self.session.headers['Authorization'] = f'Bearer {token}'
        return token
    
    def check_api(self) -> Dict:
        """Check if the registry API is accessible"""
        response = self.session.get(f"{self.base_url}/v2/")
        response.raise_for_status()
        return response.json()
    
    def list_repositories(self, n: Optional[int] = None, last: Optional[str] = None) -> List[str]:
        """List all repositories"""
        params = {}
        if n is not None:
            params['n'] = n
        if last is not None:
            params['last'] = last
            
        response = self.session.get(f"{self.base_url}/v2/_catalog", params=params)
        response.raise_for_status()
        return response.json()["repositories"]
    
    def list_tags(self, repository: str, n: Optional[int] = None, last: Optional[str] = None) -> List[str]:
        """List tags for a repository"""
        params = {}
        if n is not None:
            params['n'] = n
        if last is not None:
            params['last'] = last
            
        response = self.session.get(f"{self.base_url}/v2/{repository}/tags/list", params=params)
        response.raise_for_status()
        return response.json()["tags"]
    
    def get_manifest(self, repository: str, tag_or_digest: str) -> Dict:
        """Get manifest for a tag or digest"""
        headers = {
            'Accept': 'application/vnd.docker.distribution.manifest.v2+json'
        }
        response = self.session.get(
            f"{self.base_url}/v2/{repository}/manifests/{tag_or_digest}",
            headers=headers
        )
        response.raise_for_status()
        return response.json()
    
    def blob_exists(self, repository: str, digest: str) -> bool:
        """Check if a blob exists"""
        response = self.session.head(f"{self.base_url}/v2/{repository}/blobs/{digest}")
        return response.status_code == 200
    
    def get_blob(self, repository: str, digest: str) -> bytes:
        """Download a blob"""
        response = self.session.get(f"{self.base_url}/v2/{repository}/blobs/{digest}")
        response.raise_for_status()
        return response.content
    
    def delete_manifest(self, repository: str, digest: str) -> bool:
        """Delete a manifest"""
        response = self.session.delete(f"{self.base_url}/v2/{repository}/manifests/{digest}")
        return response.status_code == 202
    
    # Management API methods
    def get_repository_info(self, repository: str) -> Dict:
        """Get detailed repository information"""
        response = self.session.get(f"{self.base_url}/api/v1/repositories/{repository}")
        response.raise_for_status()
        return response.json()
    
    def list_users(self, page: int = 1, limit: int = 50) -> Dict:
        """List users (admin only)"""
        params = {'page': page, 'limit': limit}
        response = self.session.get(f"{self.base_url}/api/v1/users", params=params)
        response.raise_for_status()
        return response.json()
    
    def create_token(self, name: str, permissions: List[str], expires_at: Optional[str] = None) -> Dict:
        """Create a personal access token"""
        data = {
            'name': name,
            'permissions': permissions
        }
        if expires_at:
            data['expires_at'] = expires_at
            
        response = self.session.post(f"{self.base_url}/api/v1/tokens", json=data)
        response.raise_for_status()
        return response.json()
    
    def get_health(self) -> Dict:
        """Get system health information"""
        response = self.session.get(f"{self.base_url}/api/v1/health")
        response.raise_for_status()
        return response.json()
    
    def get_metrics(self) -> Dict:
        """Get system metrics"""
        response = self.session.get(f"{self.base_url}/api/v1/metrics")
        response.raise_for_status()
        return response.json()


def main():
    """Example usage of the GhostDock client"""
    
    # Initialize client with credentials
    client = GhostDockClient(
        base_url="http://localhost:5000",
        username="admin",
        password="password"
    )
    
    try:
        # Check API connectivity
        print("Checking API...")
        api_info = client.check_api()
        print(f"Connected to {api_info.get('registry', 'GhostDock Registry')}")
        
        # List repositories
        print("\nRepositories:")
        repositories = client.list_repositories()
        for repo in repositories:
            print(f"  - {repo}")
            
            # List tags for each repository
            tags = client.list_tags(repo)
            for tag in tags[:3]:  # Show first 3 tags
                print(f"    - {tag}")
        
        # Get repository details (if any repositories exist)
        if repositories:
            repo_name = repositories[0]
            print(f"\nDetails for {repo_name}:")
            try:
                repo_info = client.get_repository_info(repo_name)
                print(f"  Size: {repo_info.get('size', 0)} bytes")
                print(f"  Pull count: {repo_info.get('pull_count', 0)}")
                print(f"  Created: {repo_info.get('created_at', 'N/A')}")
            except requests.HTTPError as e:
                print(f"  Could not get repository details: {e}")
        
        # Get system health
        print("\nSystem Health:")
        health = client.get_health()
        print(f"  Status: {health.get('status')}")
        print(f"  Version: {health.get('version')}")
        
        services = health.get('services', {})
        for service, status in services.items():
            print(f"  {service}: {status}")
        
        # Get system metrics  
        print("\nSystem Metrics:")
        metrics = client.get_metrics()
        registry_metrics = metrics.get('registry', {})
        print(f"  Repositories: {registry_metrics.get('repositories', 0)}")
        print(f"  Images: {registry_metrics.get('images', 0)}")
        print(f"  Total pulls: {registry_metrics.get('pulls', 0)}")
        print(f"  Total pushes: {registry_metrics.get('pushes', 0)}")
        
    except requests.RequestException as e:
        print(f"Error: {e}")
        return 1
    
    return 0


if __name__ == "__main__":
    exit(main())
