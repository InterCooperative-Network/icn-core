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
  Input,
  TextArea,
  Separator,
  Badge,
  Avatar,
  Sheet,
} from '@tamagui/core';
import { MessageSquare, Plus, Send, ThumbsUp, Heart, Lightbulb } from '@tamagui/lucide-icons';
import { useGovernance } from '@/hooks/useGovernance';

export default function DiscussionsScreen() {
  const { state, addDiscussion, isLoading } = useGovernance();
  const [showCreateSheet, setShowCreateSheet] = useState(false);
  const [selectedProposal, setSelectedProposal] = useState('');
  const [newDiscussion, setNewDiscussion] = useState('');
  const [filter, setFilter] = useState<'all' | 'active' | 'recent'>('all');

  const handleCreateDiscussion = async () => {
    if (!newDiscussion.trim() || !selectedProposal) {
      Alert.alert('Error', 'Please select a proposal and enter your discussion.');
      return;
    }

    try {
      await addDiscussion(selectedProposal, newDiscussion);
      Alert.alert('Success', 'Discussion added successfully!');
      setShowCreateSheet(false);
      setNewDiscussion('');
      setSelectedProposal('');
    } catch (error) {
      Alert.alert('Error', 'Failed to add discussion. Please try again.');
    }
  };

  const getProposalTitle = (proposalId: string) => {
    const proposal = state.proposals.find(p => p.id === proposalId);
    return proposal ? proposal.title : 'Unknown Proposal';
  };

  const getProposalStatus = (proposalId: string) => {
    const proposal = state.proposals.find(p => p.id === proposalId);
    return proposal ? proposal.status : 'unknown';
  };

  const filteredDiscussions = state.discussions.filter(discussion => {
    if (filter === 'all') return true;
    if (filter === 'active') {
      const proposal = state.proposals.find(p => p.id === discussion.proposal_id);
      return proposal?.status === 'active';
    }
    if (filter === 'recent') {
      const oneDayAgo = new Date(Date.now() - 24 * 60 * 60 * 1000);
      return new Date(discussion.timestamp) > oneDayAgo;
    }
    return true;
  });

  const groupedDiscussions = filteredDiscussions.reduce((acc, discussion) => {
    const proposalId = discussion.proposal_id;
    if (!acc[proposalId]) {
      acc[proposalId] = [];
    }
    acc[proposalId].push(discussion);
    return acc;
  }, {} as Record<string, typeof state.discussions>);

  return (
    <>
      <ScrollView style={{ flex: 1, backgroundColor: '#f8fafc' }}>
        <YStack padding="$4" space="$4">
          {/* Header */}
          <XStack justifyContent="space-between" alignItems="center">
            <YStack>
              <H1 color="$primary">Discussions</H1>
              <Paragraph color="$gray11">Deliberate and share perspectives</Paragraph>
            </YStack>
            <Button
              theme="blue"
              icon={Plus}
              onPress={() => setShowCreateSheet(true)}
            >
              Add Discussion
            </Button>
          </XStack>

          {/* Filter Tabs */}
          <XStack space="$2" flexWrap="wrap">
            {(['all', 'active', 'recent'] as const).map(filterOption => (
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

          {/* Discussions */}
          <YStack space="$4">
            {Object.keys(groupedDiscussions).length === 0 ? (
              <Card padding="$4" backgroundColor="$gray2">
                <YStack alignItems="center" space="$3">
                  <MessageSquare size={48} color="$gray9" />
                  <H3 color="$gray11">No discussions found</H3>
                  <Paragraph color="$gray11" textAlign="center">
                    Start the conversation! Share your thoughts on active proposals.
                  </Paragraph>
                  <Button
                    theme="blue"
                    onPress={() => setShowCreateSheet(true)}
                  >
                    Start Discussion
                  </Button>
                </YStack>
              </Card>
            ) : (
              Object.entries(groupedDiscussions).map(([proposalId, discussions]) => (
                <Card key={proposalId} padding="$4">
                  <YStack space="$4">
                    {/* Proposal Header */}
                    <YStack space="$2">
                      <XStack alignItems="center" space="$2">
                        <H3 color="$blue11" flex={1}>{getProposalTitle(proposalId)}</H3>
                        <Badge 
                          theme={
                            getProposalStatus(proposalId) === 'active' ? 'green' :
                            getProposalStatus(proposalId) === 'passed' ? 'blue' :
                            getProposalStatus(proposalId) === 'failed' ? 'red' : 'gray'
                          }
                        >
                          {getProposalStatus(proposalId)}
                        </Badge>
                      </XStack>
                      <Paragraph size="$3" color="$gray11">
                        {discussions.length} discussion{discussions.length !== 1 ? 's' : ''}
                      </Paragraph>
                    </YStack>

                    <Separator />

                    {/* Discussion Messages */}
                    <YStack space="$3">
                      {discussions
                        .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
                        .map(discussion => (
                          <Card key={discussion.id} padding="$3" backgroundColor="$gray1">
                            <YStack space="$3">
                              {/* Author and Timestamp */}
                              <XStack alignItems="center" space="$3">
                                <Avatar circular size="$3">
                                  <Avatar.Image source={{ uri: `https://api.dicebear.com/7.x/avataaars/png?seed=${discussion.author}` }} />
                                  <Avatar.Fallback backgroundColor="$blue4">
                                    <Paragraph color="$blue11">
                                      {discussion.author.charAt(0).toUpperCase()}
                                    </Paragraph>
                                  </Avatar.Fallback>
                                </Avatar>
                                <YStack flex={1}>
                                  <Paragraph fontWeight="600" size="$4">
                                    {discussion.author}
                                  </Paragraph>
                                  <Paragraph size="$3" color="$gray11">
                                    {new Date(discussion.timestamp).toLocaleDateString()} at{' '}
                                    {new Date(discussion.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                                  </Paragraph>
                                </YStack>
                              </XStack>

                              {/* Content */}
                              <Paragraph color="$gray12">{discussion.content}</Paragraph>

                              {/* Reactions */}
                              {Object.keys(discussion.reactions).length > 0 && (
                                <XStack space="$2" flexWrap="wrap">
                                  {Object.entries(discussion.reactions).map(([emoji, users]) => (
                                    <Badge key={emoji} theme="gray" size="$2">
                                      {emoji} {users.length}
                                    </Badge>
                                  ))}
                                </XStack>
                              )}

                              {/* Quick Reactions */}
                              <XStack space="$2">
                                <Button size="$2" theme="gray" icon={ThumbsUp}>
                                  Like
                                </Button>
                                <Button size="$2" theme="gray" icon={Heart}>
                                  Support
                                </Button>
                                <Button size="$2" theme="gray" icon={Lightbulb}>
                                  Insight
                                </Button>
                              </XStack>
                            </YStack>
                          </Card>
                        ))}
                    </YStack>
                  </YStack>
                </Card>
              ))
            )}
          </YStack>
        </YStack>
      </ScrollView>

      {/* Create Discussion Sheet */}
      <Sheet
        modal
        open={showCreateSheet}
        onOpenChange={setShowCreateSheet}
        snapPoints={[80]}
        dismissOnSnapToBottom
      >
        <Sheet.Overlay />
        <Sheet.Handle />
        <Sheet.Frame padding="$4" backgroundColor="$background">
          <ScrollView>
            <YStack space="$4">
              <H2>Add Discussion</H2>
              
              <YStack space="$3">
                <YStack space="$2">
                  <Paragraph fontWeight="600">Select Proposal *</Paragraph>
                  <YStack space="$2">
                    {state.proposals.map(proposal => (
                      <Button
                        key={proposal.id}
                        theme={selectedProposal === proposal.id ? 'blue' : 'gray'}
                        onPress={() => setSelectedProposal(proposal.id)}
                        justifyContent="flex-start"
                        textAlign="left"
                      >
                        <YStack alignItems="flex-start" space="$1">
                          <Paragraph fontWeight="600">{proposal.title}</Paragraph>
                          <Badge 
                            theme={
                              proposal.status === 'active' ? 'green' :
                              proposal.status === 'passed' ? 'blue' :
                              proposal.status === 'failed' ? 'red' : 'gray'
                            }
                          >
                            {proposal.status}
                          </Badge>
                        </YStack>
                      </Button>
                    ))}
                  </YStack>
                </YStack>

                <YStack space="$2">
                  <Paragraph fontWeight="600">Your Discussion *</Paragraph>
                  <TextArea
                    value={newDiscussion}
                    onChangeText={setNewDiscussion}
                    placeholder="Share your thoughts, concerns, or insights about this proposal..."
                    minHeight={120}
                  />
                </YStack>

                <Card padding="$3" backgroundColor="$blue1">
                  <YStack space="$2">
                    <Paragraph fontWeight="600" color="$blue11">Discussion Guidelines</Paragraph>
                    <YStack space="$1">
                      <Paragraph size="$3" color="$blue11">• Be respectful and constructive</Paragraph>
                      <Paragraph size="$3" color="$blue11">• Focus on the proposal's merits</Paragraph>
                      <Paragraph size="$3" color="$blue11">• Provide evidence for claims</Paragraph>
                      <Paragraph size="$3" color="$blue11">• Consider diverse perspectives</Paragraph>
                    </YStack>
                  </YStack>
                </Card>
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
                  icon={Send}
                  onPress={handleCreateDiscussion}
                  disabled={isLoading}
                >
                  {isLoading ? 'Posting...' : 'Post Discussion'}
                </Button>
              </XStack>
            </YStack>
          </ScrollView>
        </Sheet.Frame>
      </Sheet>
    </>
  );
}