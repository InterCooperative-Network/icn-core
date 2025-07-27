import React, { useState } from 'react';
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
  Avatar,
  Badge,
  Separator,
  Progress,
} from '@tamagui/core';
import { Users, Star, Award, Calendar, TrendingUp, UserPlus } from '@tamagui/lucide-icons';
import { useGovernance } from '@/hooks/useGovernance';

export default function CommunityScreen() {
  const { state } = useGovernance();
  const [filter, setFilter] = useState<'all' | 'active' | 'delegates' | 'newcomers'>('all');

  const membersWithStats = state.members.map(member => {
    const memberVotes = state.votes.filter(vote => vote.voter === member.id);
    const memberDiscussions = state.discussions.filter(discussion => discussion.author === member.name);
    
    return {
      ...member,
      total_votes: memberVotes.length,
      total_discussions: memberDiscussions.length,
      activity_score: memberVotes.length * 2 + memberDiscussions.length,
    };
  });

  const filteredMembers = membersWithStats.filter(member => {
    if (filter === 'all') return true;
    if (filter === 'active') return member.activity_score > 0;
    if (filter === 'delegates') return member.delegation;
    if (filter === 'newcomers') {
      const joinedRecently = new Date(member.joined_at) > new Date(Date.now() - 30 * 24 * 60 * 60 * 1000);
      return joinedRecently;
    }
    return true;
  });

  const totalParticipation = state.votes.length + state.discussions.length;
  const activeMembers = membersWithStats.filter(m => m.activity_score > 0).length;

  const upcomingEvents = [
    {
      id: '1',
      title: 'Community Call: Q1 Planning',
      date: '2024-01-25T18:00:00Z',
      type: 'Meeting',
      attendees: 15,
    },
    {
      id: '2',
      title: 'Governance Workshop',
      date: '2024-01-28T14:00:00Z',
      type: 'Workshop',
      attendees: 8,
    },
    {
      id: '3',
      title: 'Proposal Review Session',
      date: '2024-01-30T16:00:00Z',
      type: 'Review',
      attendees: 12,
    },
  ];

  return (
    <ScrollView style={{ flex: 1, backgroundColor: '#f8fafc' }}>
      <YStack padding="$4" space="$4">
        {/* Header */}
        <YStack space="$2">
          <H1 color="$primary">Community</H1>
          <Paragraph color="$gray11">Connect with fellow cooperative members</Paragraph>
        </YStack>

        {/* Community Stats */}
        <Card padding="$4" backgroundColor="$purple2">
          <YStack space="$3">
            <H2 color="$purple11">Community Overview</H2>
            <XStack space="$4" flexWrap="wrap">
              <YStack alignItems="center" minWidth={80}>
                <H3 color="$purple11">{state.members.length}</H3>
                <Paragraph size="$3" color="$gray11">Total Members</Paragraph>
              </YStack>
              <YStack alignItems="center" minWidth={80}>
                <H3 color="$green11">{activeMembers}</H3>
                <Paragraph size="$3" color="$gray11">Active Members</Paragraph>
              </YStack>
              <YStack alignItems="center" minWidth={80}>
                <H3 color="$blue11">{totalParticipation}</H3>
                <Paragraph size="$3" color="$gray11">Total Participation</Paragraph>
              </YStack>
              <YStack alignItems="center" minWidth={80}>
                <H3 color="$orange11">{Math.round((activeMembers / state.members.length) * 100)}%</H3>
                <Paragraph size="$3" color="$gray11">Engagement Rate</Paragraph>
              </YStack>
            </XStack>
          </YStack>
        </Card>

        {/* Upcoming Events */}
        <Card padding="$4">
          <YStack space="$3">
            <XStack alignItems="center" space="$2">
              <Calendar size={20} color="$blue9" />
              <H2>Upcoming Events</H2>
            </XStack>
            <YStack space="$2">
              {upcomingEvents.map(event => (
                <Card key={event.id} padding="$3" backgroundColor="$blue1" hoverTheme pressTheme>
                  <XStack justifyContent="space-between" alignItems="center">
                    <YStack flex={1} space="$1">
                      <Paragraph fontWeight="600">{event.title}</Paragraph>
                      <XStack alignItems="center" space="$2">
                        <Badge theme="blue" size="$2">{event.type}</Badge>
                        <Paragraph size="$3" color="$gray11">
                          {new Date(event.date).toLocaleDateString()} at{' '}
                          {new Date(event.date).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                        </Paragraph>
                      </XStack>
                    </YStack>
                    <YStack alignItems="center">
                      <Paragraph size="$3" color="$gray11">{event.attendees}</Paragraph>
                      <Paragraph size="$2" color="$gray11">attending</Paragraph>
                    </YStack>
                  </XStack>
                </Card>
              ))}
            </YStack>
            <Button theme="blue" size="$3">
              View All Events
            </Button>
          </YStack>
        </Card>

        {/* Member Filter */}
        <YStack space="$3">
          <XStack justifyContent="space-between" alignItems="center">
            <H2>Members</H2>
            <Button theme="green" icon={UserPlus} size="$3">
              Invite Member
            </Button>
          </XStack>

          <XStack space="$2" flexWrap="wrap">
            {(['all', 'active', 'delegates', 'newcomers'] as const).map(filterOption => (
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
        </YStack>

        {/* Members List */}
        <YStack space="$3">
          {filteredMembers.length === 0 ? (
            <Card padding="$4" backgroundColor="$gray2">
              <YStack alignItems="center" space="$3">
                <Users size={48} color="$gray9" />
                <H3 color="$gray11">No members found</H3>
                <Paragraph color="$gray11" textAlign="center">
                  No members match the current filter.
                </Paragraph>
              </YStack>
            </Card>
          ) : (
            filteredMembers
              .sort((a, b) => b.activity_score - a.activity_score)
              .map(member => (
                <Card key={member.id} padding="$4" hoverTheme pressTheme>
                  <YStack space="$3">
                    {/* Member Header */}
                    <XStack alignItems="center" space="$3">
                      <Avatar circular size="$5">
                        <Avatar.Image 
                          source={{ 
                            uri: member.avatar || `https://api.dicebear.com/7.x/avataaars/png?seed=${member.name}` 
                          }} 
                        />
                        <Avatar.Fallback backgroundColor="$blue4">
                          <Paragraph color="$blue11" fontSize="$6">
                            {member.name.charAt(0).toUpperCase()}
                          </Paragraph>
                        </Avatar.Fallback>
                      </Avatar>
                      
                      <YStack flex={1} space="$1">
                        <XStack alignItems="center" space="$2">
                          <H3>{member.name}</H3>
                          {member.activity_score > 10 && (
                            <Badge theme="gold" size="$2">
                              <Award size={12} />
                              Top Contributor
                            </Badge>
                          )}
                          {member.delegation && (
                            <Badge theme="purple" size="$2">
                              Delegate
                            </Badge>
                          )}
                        </XStack>
                        <Paragraph size="$3" color="$gray11">
                          Member since {new Date(member.joined_at).toLocaleDateString()}
                        </Paragraph>
                      </YStack>
                      
                      <YStack alignItems="flex-end" space="$1">
                        <XStack alignItems="center" space="$1">
                          <Star size={14} color="$orange9" />
                          <Paragraph fontWeight="600">{member.reputation}</Paragraph>
                        </XStack>
                        <Paragraph size="$3" color="$gray11">reputation</Paragraph>
                      </YStack>
                    </XStack>

                    <Separator />

                    {/* Member Stats */}
                    <YStack space="$2">
                      <XStack justifyContent="space-between" alignItems="center">
                        <Paragraph size="$4" fontWeight="600">Activity Overview</Paragraph>
                        <Badge theme="blue" size="$2">
                          Score: {member.activity_score}
                        </Badge>
                      </XStack>
                      
                      <XStack space="$4" flexWrap="wrap">
                        <YStack alignItems="center" minWidth={80}>
                          <Paragraph fontWeight="600" color="$blue11">{member.total_votes}</Paragraph>
                          <Paragraph size="$3" color="$gray11">Votes Cast</Paragraph>
                        </YStack>
                        <YStack alignItems="center" minWidth={80}>
                          <Paragraph fontWeight="600" color="$green11">{member.total_discussions}</Paragraph>
                          <Paragraph size="$3" color="$gray11">Discussions</Paragraph>
                        </YStack>
                        <YStack alignItems="center" minWidth={80}>
                          <Paragraph fontWeight="600" color="$purple11">{member.voting_power.toFixed(1)}</Paragraph>
                          <Paragraph size="$3" color="$gray11">Voting Power</Paragraph>
                        </YStack>
                      </XStack>

                      {/* Activity Progress */}
                      <YStack space="$1">
                        <XStack justifyContent="space-between">
                          <Paragraph size="$3" color="$gray11">Participation Level</Paragraph>
                          <Paragraph size="$3" color="$gray11">
                            {Math.min(100, Math.round((member.activity_score / 20) * 100))}%
                          </Paragraph>
                        </XStack>
                        <Progress 
                          value={Math.min(100, (member.activity_score / 20) * 100)} 
                          backgroundColor="$gray4"
                        >
                          <Progress.Indicator 
                            animation="bouncy" 
                            backgroundColor={
                              member.activity_score > 15 ? '$green9' :
                              member.activity_score > 10 ? '$blue9' :
                              member.activity_score > 5 ? '$orange9' : '$gray9'
                            } 
                          />
                        </Progress>
                      </YStack>
                    </YStack>

                    {/* Quick Actions */}
                    <XStack space="$2">
                      <Button flex={1} theme="blue" size="$3">
                        View Profile
                      </Button>
                      <Button flex={1} theme="gray" size="$3">
                        Message
                      </Button>
                    </XStack>
                  </YStack>
                </Card>
              ))
          )}
        </YStack>

        {/* Community Guidelines */}
        <Card padding="$4" backgroundColor="$green1">
          <YStack space="$3">
            <XStack alignItems="center" space="$2">
              <Award size={20} color="$green9" />
              <H3 color="$green11">Community Guidelines</H3>
            </XStack>
            <YStack space="$2">
              <Paragraph color="$green11">
                • Participate respectfully in discussions and voting
              </Paragraph>
              <Paragraph color="$green11">
                • Share knowledge and help newcomers learn
              </Paragraph>
              <Paragraph color="$green11">
                • Consider the collective benefit in all decisions
              </Paragraph>
              <Paragraph color="$green11">
                • Attend community events when possible
              </Paragraph>
            </YStack>
          </YStack>
        </Card>
      </YStack>
    </ScrollView>
  );
}