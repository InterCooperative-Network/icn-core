import React, { useState } from 'react';
import { ScrollView, Alert } from 'react-native';
import {
  YStack,
  XStack,
  Card,
  H1,
  H2,
  H3,
  Paragraph,
  Button,
  Progress,
  Separator,
  Badge,
  Input,
  TextArea,
  Sheet,
  Switch,
} from '@tamagui/core';
import { Plus, Vote, Clock, User, ChevronDown } from '@tamagui/lucide-icons';
import { useGovernance } from '@/hooks/useGovernance';
import { Proposal } from '@/types/governance';

export default function ProposalsScreen() {
  const { state, submitProposal, vote, isLoading } = useGovernance();
  const [showCreateSheet, setShowCreateSheet] = useState(false);
  const [filter, setFilter] = useState<'all' | 'active' | 'draft' | 'completed'>('all');
  const [newProposal, setNewProposal] = useState({
    title: '',
    description: '',
    author: state.current_user?.name || 'Anonymous',
    status: 'draft' as const,
    voting_deadline: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString(),
    quorum_required: 0.51,
    threshold_required: 0.51,
  });

  const filteredProposals = state.proposals.filter(proposal => {
    if (filter === 'all') return true;
    if (filter === 'active') return proposal.status === 'active';
    if (filter === 'draft') return proposal.status === 'draft';
    if (filter === 'completed') return ['passed', 'failed', 'executed'].includes(proposal.status);
    return true;
  });

  const handleVote = async (proposalId: string, choice: 'yes' | 'no' | 'abstain') => {
    try {
      await vote(proposalId, choice);
      Alert.alert('Success', 'Your vote has been recorded!');
    } catch (error) {
      Alert.alert('Error', 'Failed to submit vote. Please try again.');
    }
  };

  const handleCreateProposal = async () => {
    if (!newProposal.title.trim() || !newProposal.description.trim()) {
      Alert.alert('Error', 'Please fill in all required fields.');
      return;
    }

    try {
      await submitProposal(newProposal);
      Alert.alert('Success', 'Proposal created successfully!');
      setShowCreateSheet(false);
      setNewProposal({
        title: '',
        description: '',
        author: state.current_user?.name || 'Anonymous',
        status: 'draft',
        voting_deadline: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString(),
        quorum_required: 0.51,
        threshold_required: 0.51,
      });
    } catch (error) {
      Alert.alert('Error', 'Failed to create proposal. Please try again.');
    }
  };

  const getUserVote = (proposalId: string) => {
    return state.votes.find(v => v.proposal_id === proposalId && v.voter === state.current_user?.id);
  };

  return (
    <>
      <ScrollView style={{ flex: 1, backgroundColor: '#f8fafc' }}>
        <YStack padding="$4" space="$4">
          {/* Header */}
          <XStack justifyContent="space-between" alignItems="center">
            <YStack>
              <H1 color="$primary">Proposals</H1>
              <Paragraph color="$gray11">Participate in democratic governance</Paragraph>
            </YStack>
            <Button
              theme="blue"
              icon={Plus}
              onPress={() => setShowCreateSheet(true)}
            >
              Create
            </Button>
          </XStack>

          {/* Filter Tabs */}
          <XStack space="$2" flexWrap="wrap">
            {(['all', 'active', 'draft', 'completed'] as const).map(filterOption => (
              <Button
                key={filterOption}
                size="$3"
                theme={filter === filterOption ? 'blue' : 'gray'}
                onPress={() => setFilter(filterOption)}
              >
                {filterOption.charAt(0).toUpperCase() + filterOption.slice(1)}
              </Button>
            ))}
          </XStack>

          {/* Proposals List */}
          <YStack space="$3">
            {filteredProposals.length === 0 ? (
              <Card padding="$4" backgroundColor="$gray2">
                <YStack alignItems="center" space="$3">
                  <Vote size={48} color="$gray9" />
                  <H3 color="$gray11">No proposals found</H3>
                  <Paragraph color="$gray11" textAlign="center">
                    {filter === 'all' 
                      ? 'Be the first to create a proposal for the community!'
                      : `No ${filter} proposals at the moment.`
                    }
                  </Paragraph>
                  <Button
                    theme="blue"
                    onPress={() => setShowCreateSheet(true)}
                  >
                    Create Proposal
                  </Button>
                </YStack>
              </Card>
            ) : (
              filteredProposals.map(proposal => {
                const userVote = getUserVote(proposal.id);
                const totalVotes = proposal.votes.yes + proposal.votes.no + proposal.votes.abstain;
                const yesPercentage = totalVotes > 0 ? (proposal.votes.yes / totalVotes) * 100 : 0;

                return (
                  <Card key={proposal.id} padding="$4" hoverTheme pressTheme>
                    <YStack space="$4">
                      {/* Header */}
                      <XStack justifyContent="space-between" alignItems="flex-start">
                        <YStack flex={1} space="$2">
                          <XStack alignItems="center" space="$2" flexWrap="wrap">
                            <H3 color="$blue11" flex={1}>{proposal.title}</H3>
                            <Badge 
                              theme={
                                proposal.status === 'active' ? 'green' :
                                proposal.status === 'passed' ? 'blue' :
                                proposal.status === 'failed' ? 'red' : 'gray'
                              }
                            >
                              {proposal.status}
                            </Badge>
                          </XStack>
                          <XStack alignItems="center" space="$2">
                            <User size={14} color="$gray9" />
                            <Paragraph size="$3" color="$gray11">{proposal.author}</Paragraph>
                            <Clock size={14} color="$gray9" />
                            <Paragraph size="$3" color="$gray11">
                              Deadline: {new Date(proposal.voting_deadline).toLocaleDateString()}
                            </Paragraph>
                          </XStack>
                        </YStack>
                      </XStack>

                      {/* Description */}
                      <Paragraph color="$gray12">{proposal.description}</Paragraph>

                      {/* Changes (if any) */}
                      {proposal.changes && proposal.changes.length > 0 && (
                        <YStack space="$2">
                          <Paragraph fontWeight="600" size="$4">Proposed Changes:</Paragraph>
                          {proposal.changes.map((change, index) => (
                            <Card key={index} padding="$3" backgroundColor="$blue1">
                              <YStack space="$1">
                                <Paragraph fontWeight="600" size="$3">{change.parameter}</Paragraph>
                                <XStack space="$2" alignItems="center">
                                  <Paragraph size="$3" color="$red10">
                                    Current: {String(change.current_value)}
                                  </Paragraph>
                                  <Paragraph size="$3">→</Paragraph>
                                  <Paragraph size="$3" color="$green10">
                                    Proposed: {String(change.proposed_value)}
                                  </Paragraph>
                                </XStack>
                                {change.description && (
                                  <Paragraph size="$3" color="$gray11">{change.description}</Paragraph>
                                )}
                              </YStack>
                            </Card>
                          ))}
                        </YStack>
                      )}

                      <Separator />

                      {/* Voting Progress */}
                      <YStack space="$3">
                        <XStack justifyContent="space-between" alignItems="center">
                          <Paragraph fontWeight="600">Voting Progress</Paragraph>
                          <Paragraph size="$3" color="$gray11">
                            {totalVotes} votes • Quorum: {(proposal.quorum_required * 100).toFixed(0)}%
                          </Paragraph>
                        </XStack>
                        
                        <Progress value={yesPercentage} backgroundColor="$gray4">
                          <Progress.Indicator animation="bouncy" backgroundColor="$green9" />
                        </Progress>
                        
                        <XStack justifyContent="space-between" space="$2">
                          <Badge theme="green" flex={1} justifyContent="center">
                            {proposal.votes.yes} Yes ({totalVotes > 0 ? Math.round((proposal.votes.yes / totalVotes) * 100) : 0}%)
                          </Badge>
                          <Badge theme="red" flex={1} justifyContent="center">
                            {proposal.votes.no} No ({totalVotes > 0 ? Math.round((proposal.votes.no / totalVotes) * 100) : 0}%)
                          </Badge>
                          <Badge theme="gray" flex={1} justifyContent="center">
                            {proposal.votes.abstain} Abstain
                          </Badge>
                        </XStack>
                      </YStack>

                      {/* Voting Buttons */}
                      {proposal.status === 'active' && (
                        <YStack space="$3">
                          {userVote ? (
                            <Card padding="$3" backgroundColor="$blue1">
                              <Paragraph textAlign="center" color="$blue11">
                                You voted: {userVote.choice.toUpperCase()}
                              </Paragraph>
                            </Card>
                          ) : (
                            <XStack space="$2">
                              <Button
                                flex={1}
                                theme="green"
                                onPress={() => handleVote(proposal.id, 'yes')}
                                disabled={isLoading}
                              >
                                Vote Yes
                              </Button>
                              <Button
                                flex={1}
                                theme="red"
                                onPress={() => handleVote(proposal.id, 'no')}
                                disabled={isLoading}
                              >
                                Vote No
                              </Button>
                              <Button
                                flex={1}
                                theme="gray"
                                onPress={() => handleVote(proposal.id, 'abstain')}
                                disabled={isLoading}
                              >
                                Abstain
                              </Button>
                            </XStack>
                          )}
                        </YStack>
                      )}
                    </YStack>
                  </Card>
                );
              })
            )}
          </YStack>
        </YStack>
      </ScrollView>

      {/* Create Proposal Sheet */}
      <Sheet
        modal
        open={showCreateSheet}
        onOpenChange={setShowCreateSheet}
        snapPoints={[90]}
        dismissOnSnapToBottom
      >
        <Sheet.Overlay />
        <Sheet.Handle />
        <Sheet.Frame padding="$4" backgroundColor="$background">
          <ScrollView>
            <YStack space="$4">
              <H2>Create New Proposal</H2>
              
              <YStack space="$3">
                <YStack space="$2">
                  <Paragraph fontWeight="600">Title *</Paragraph>
                  <Input
                    value={newProposal.title}
                    onChangeText={(text) => setNewProposal(prev => ({ ...prev, title: text }))}
                    placeholder="Enter proposal title"
                  />
                </YStack>

                <YStack space="$2">
                  <Paragraph fontWeight="600">Description *</Paragraph>
                  <TextArea
                    value={newProposal.description}
                    onChangeText={(text) => setNewProposal(prev => ({ ...prev, description: text }))}
                    placeholder="Describe your proposal in detail..."
                    minHeight={120}
                  />
                </YStack>

                <YStack space="$2">
                  <Paragraph fontWeight="600">Voting Period (days)</Paragraph>
                  <Input
                    keyboardType="numeric"
                    value="7"
                    placeholder="7"
                    editable={false}
                  />
                </YStack>

                <XStack space="$4" alignItems="center">
                  <Paragraph fontWeight="600">Start as Draft</Paragraph>
                  <Switch
                    checked={newProposal.status === 'draft'}
                    onCheckedChange={(checked) => 
                      setNewProposal(prev => ({ 
                        ...prev, 
                        status: checked ? 'draft' : 'active' 
                      }))
                    }
                  />
                </XStack>
              </YStack>

              <XStack space="$3">
                <Button
                  flex={1}
                  theme="gray"
                  onPress={() => setShowCreateSheet(false)}
                >
                  Cancel
                </Button>
                <Button
                  flex={1}
                  theme="blue"
                  onPress={handleCreateProposal}
                  disabled={isLoading}
                >
                  {isLoading ? 'Creating...' : 'Create Proposal'}
                </Button>
              </XStack>
            </YStack>
          </ScrollView>
        </Sheet.Frame>
      </Sheet>
    </>
  );
}