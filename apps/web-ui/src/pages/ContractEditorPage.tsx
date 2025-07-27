import React from 'react'
import { VisualEditor } from '@icn/ccl-visual-editor'
import { useICNClient } from '@icn/ts-sdk'
import type { CCLGenerationResult } from '@icn/ccl-visual-editor'

export function ContractEditorPage() {
  const icnClient = useICNClient()

  const handleCodeGenerated = (result: CCLGenerationResult) => {
    console.log('Generated CCL code:', result)
  }

  const handleContractDeploy = async (cclCode: string) => {
    try {
      console.log('Deploying contract with code:', cclCode)
      
      // Call the ICN node contracts endpoint
      const response = await fetch(`${icnClient.getConnectionState().nodeEndpoint}/contracts`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          source: cclCode
        })
      })

      if (!response.ok) {
        throw new Error(`Deployment failed: ${response.statusText}`)
      }

      const result = await response.json()
      console.log('Contract deployed successfully:', result)
      
      // Could show success notification here
      alert(`Contract deployed successfully! CID: ${result.manifest_cid}`)
    } catch (error) {
      console.error('Contract deployment failed:', error)
      alert(`Deployment failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
      throw error
    }
  }

  return (
    <div className="h-screen -m-8 -mt-8"> {/* Negative margins to make it full screen */}
      <VisualEditor
        onCodeGenerated={handleCodeGenerated}
        onContractDeploy={handleContractDeploy}
        className="h-full"
      />
    </div>
  )
}