import React from 'react';
import { ScrollView } from 'react-native';
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
} from '@tamagui/core';
import { Vote, Clock, Users, TrendingUp } from '@tamagui/lucide-icons';
import { useGovernance } from '@/hooks/useGovernance';
import { useRouter } from 'expo-router';

export default function HomeScreen() {
  const { state } = useGovernance();
  const router = useRouter();

  const activeProposals = state.proposals.filter(p => p.status === 'active');
  const totalVotes = state.proposals.reduce((sum, p) => sum + p.votes.yes + p.votes.no + p.votes.abstain, 0);

  return (
    <ScrollView style={{ flex: 1, backgroundColor: '#f8fafc' }}>
      <YStack padding="$4" space="$4">
        {/* Header */}
        <YStack space="$2">
          <H1 color="$primary">ICN AgoraNet</H1>
          <Paragraph color="$gray11" size="$5">
            Democratic governance for the InterCooperative Network
          </Paragraph>
        </YStack>

        {/* Stats Overview */}
        <Card padding="$4" backgroundColor="$blue2">
          <YStack space="$3">
            <H2 color="$blue11">Governance Overview</H2>
            <XStack space="$4" flexWrap="wrap">
              <YStack alignItems="center" minWidth={80}>
                <H3 color="$blue11">{state.proposals.length}</H3>
                <Paragraph size="$3" color="$gray11">Total Proposals</Paragraph>
              </YStack>
              <YStack alignItems="center" minWidth={80}>
                <H3 color="$green11">{activeProposals.length}</H3>
                <Paragraph size="$3" color="$gray11">Active Votes</Paragraph>
              </YStack>
              <YStack alignItems="center" minWidth={80}>
                <H3 color="$purple11">{state.members.length}</H3>
                <Paragraph size="$3" color="$gray11">Members</Paragraph>
              </YStack>
              <YStack alignItems="center" minWidth={80}>
                <H3 color="$orange11">{totalVotes}</H3>
                <Paragraph size="$3" color="$gray11">Total Votes</Paragraph>
              </YStack>
            </XStack>
          </YStack>
        </Card>

        {/* Active Proposals */}
        <YStack space="$3">
          <XStack justifyContent="space-between" alignItems="center">
            <H2>Active Proposals</H2>
            <Button
              size="$3"
              theme="blue"
              onPress={() => router.push('/(tabs)/proposals')}
            >
              View All
            </Button>
          </XStack>

          {activeProposals.length === 0 ? (
            <Card padding="$4" backgroundColor="$gray2">
              <YStack alignItems="center" space="$2">
                <Vote size={48} color="$gray9" />
                <Paragraph color="$gray11">No active proposals</Paragraph>
                <Button
                  size="$3"
                  theme="blue"
                  onPress={() => router.push('/(tabs)/proposals')}
                >
                  Create Proposal
                </Button>
              </YStack>
            </Card>
          ) : (
            activeProposals.slice(0, 3).map(proposal => (
              <Card key={proposal.id} padding="$4" hoverTheme pressTheme>
                <YStack space="$3">
                  <XStack justifyContent="space-between" alignItems="flex-start">
                    <YStack flex={1} space="$1">
                      <XStack alignItems="center" space="$2">
                        <H3 color="$blue11">{proposal.title}</H3>
                        <Badge theme="blue">{proposal.status}</Badge>
                      </XStack>
                      <Paragraph color="$gray11" numberOfLines={2}>
                        {proposal.description}
                      </Paragraph>
                    </YStack>
                  </XStack>

                  <Separator />

                  <YStack space="$2">
                    <XStack justifyContent="space-between" alignItems="center">
                      <Paragraph size="$3" color="$gray11">Progress</Paragraph>
                      <XStack alignItems="center" space="$1">
                        <Clock size={14} color="$gray9" />
                        <Paragraph size="$3" color="$gray11">
                          {new Date(proposal.voting_deadline).toLocaleDateString()}
                        </Paragraph>
                      </XStack>
                    </XStack>
                    
                    <Progress
                      value={(() => {
                        const totalVotes = proposal.votes.yes + proposal.votes.no + proposal.votes.abstain;
                        return totalVotes > 0 ? (proposal.votes.yes / totalVotes) * 100 : 0;
                      })()}
                      backgroundColor="$gray4"
                    >
                      <Progress.Indicator animation="bouncy" backgroundColor="$green9" />
                    </Progress>
                    
                    <XStack justifyContent="space-between" space="$2">
                      <Badge theme="green">{proposal.votes.yes} Yes</Badge>
                      <Badge theme="red">{proposal.votes.no} No</Badge>
                      <Badge theme="gray">{proposal.votes.abstain} Abstain</Badge>
                    </XStack>
                  </YStack>

                  <Button
                    theme="blue"
                    size="$3"
                    onPress={() => router.push(`/(tabs)/proposals?id=${proposal.id}`)}
                  >
                    View Details
                  </Button>
                </YStack>
              </Card>
            ))
          )}
        </YStack>

        {/* Quick Actions */}
        <Card padding="$4">
          <YStack space="$3">
            <H2>Quick Actions</H2>
            <XStack space="$3" flexWrap="wrap">
              <Button
                flex={1}
                minWidth={140}
                theme="blue"
                icon={Vote}
                onPress={() => router.push('/(tabs)/proposals')}
              >
                Create Proposal
              </Button>
              <Button
                flex={1}
                minWidth={140}
                theme="green"
                icon={Users}
                onPress={() => router.push('/(tabs)/community')}
              >
                View Community
              </Button>
            </XStack>
          </YStack>
        </Card>

        {/* Recent Activity */}
        <Card padding="$4">
          <YStack space="$3">
            <H2>Recent Activity</H2>
            <YStack space="$2">
              {state.discussions.slice(0, 3).map(discussion => (
                <XStack key={discussion.id} space="$3" alignItems="flex-start">
                  <YStack
                    width={8}
                    height={8}
                    borderRadius={4}
                    backgroundColor="$blue9"
                    marginTop="$1"
                  />
                  <YStack flex={1}>
                    <Paragraph size="$3" fontWeight="600">
                      {discussion.author} commented on a proposal
                    </Paragraph>
                    <Paragraph size="$3" color="$gray11" numberOfLines={2}>
                      {discussion.content}
                    </Paragraph>
                    <Paragraph size="$2" color="$gray9">
                      {new Date(discussion.timestamp).toLocaleDateString()}
                    </Paragraph>
                  </YStack>
                </XStack>
              ))}
              {state.discussions.length === 0 && (
                <Paragraph color="$gray11" textAlign="center">
                  No recent activity
                </Paragraph>
              )}
            </YStack>
          </YStack>
        </Card>
      </YStack>
    </ScrollView>
  );
}